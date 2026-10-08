/* Read-only EG4 ENC interoperability probe; sends one holding-register read.
 *
 * Recovered from EG4 Monitor 1.6.4:
 *   com.nfcx.eg4.tls.PSK_TLS_CONSTANT
 *   com.nfcx.eg4.tool.DonglePskUtil.calcPsk
 *   com.nfcx.eg4.tls.PskTlsConfig
 *
 * Build: cc -Wall -Wextra -O2 enc_read_probe.c -o enc_read_probe -lssl -lcrypto
 * Run: ./enc_read_probe HOST DONGLE_SERIAL INVERTER_SERIAL
 *
 * Authentication material is derived in memory and never logged. The source
 * contains no update, write-register, or configuration-changing operations.
 */
#include <arpa/inet.h>
#include <netdb.h>
#include <openssl/err.h>
#include <openssl/hmac.h>
#include <openssl/ssl.h>
#include <signal.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/time.h>
#include <unistd.h>

static unsigned char psk[16];

static unsigned int psk_callback(SSL *ssl, const char *hint, char *identity,
                                unsigned int identity_capacity,
                                unsigned char *key, unsigned int key_capacity) {
    static const char name[] = "Client_identity";
    (void)ssl;
    (void)hint;
    if (identity_capacity < sizeof(name) || key_capacity < sizeof(psk)) return 0;
    memcpy(identity, name, sizeof(name));
    memcpy(key, psk, sizeof(psk));
    return sizeof(psk);
}

static uint16_t crc16(const unsigned char *bytes, size_t count) {
    uint16_t crc = 0xffff;
    while (count--) {
        crc ^= *bytes++;
        for (int bit = 0; bit < 8; bit++)
            crc = (crc & 1) ? (crc >> 1) ^ 0xa001 : crc >> 1;
    }
    return crc;
}

static int valid_serial(const char *serial) {
    if (strlen(serial) != 10) return 0;
    for (size_t i = 0; i < 10; i++) {
        unsigned char c = (unsigned char)serial[i];
        if (!((c >= '0' && c <= '9') || (c >= 'A' && c <= 'Z') ||
              (c >= 'a' && c <= 'z'))) return 0;
    }
    return 1;
}

int main(int argc, char **argv) {
    if (argc != 4 || !valid_serial(argv[2]) || !valid_serial(argv[3])) {
        fprintf(stderr, "Usage: %s HOST DONGLE_SERIAL INVERTER_SERIAL\n", argv[0]);
        return 2;
    }
    /* Bound the entire experiment, including connection and TLS negotiation. */
    alarm(15);
    unsigned char digest[EVP_MAX_MD_SIZE];
    unsigned int digest_length = 0;
    static const char salt[] = "LuxPowerTek!";
    if (!HMAC(EVP_sha256(), salt, sizeof(salt) - 1,
              (unsigned char *)argv[2], strlen(argv[2]), digest, &digest_length)
        || digest_length < sizeof(psk)) return 1;
    memcpy(psk, digest, sizeof(psk));
    OPENSSL_cleanse(digest, sizeof(digest));

    struct addrinfo hints = {0}, *addresses = NULL;
    hints.ai_family = AF_UNSPEC;
    hints.ai_socktype = SOCK_STREAM;
    if (getaddrinfo(argv[1], "8000", &hints, &addresses) != 0) return 1;
    int fd = -1;
    struct timeval timeout = {.tv_sec = 5};
    for (struct addrinfo *a = addresses; a; a = a->ai_next) {
        fd = socket(a->ai_family, a->ai_socktype, a->ai_protocol);
        if (fd < 0) continue;
        setsockopt(fd, SOL_SOCKET, SO_RCVTIMEO, &timeout, sizeof(timeout));
        setsockopt(fd, SOL_SOCKET, SO_SNDTIMEO, &timeout, sizeof(timeout));
        if (connect(fd, a->ai_addr, a->ai_addrlen) == 0) break;
        close(fd);
        fd = -1;
    }
    freeaddrinfo(addresses);
    if (fd < 0) { perror("TCP connect"); return 1; }

    SSL_CTX *context = SSL_CTX_new(TLS_client_method());
    if (!context) { close(fd); return 1; }
    SSL_CTX_set_min_proto_version(context, TLS1_2_VERSION);
    SSL_CTX_set_max_proto_version(context, TLS1_2_VERSION);
    SSL_CTX_set_psk_client_callback(context, psk_callback);
    if (!SSL_CTX_set_cipher_list(context, "DHE-PSK-AES128-GCM-SHA256")) return 1;
    SSL *tls = SSL_new(context);
    if (!tls || SSL_set_fd(tls, fd) != 1) return 1;
    int result = SSL_connect(tls);
    if (result != 1) {
        fprintf(stderr, "TLS handshake failed (SSL error %d)\n", SSL_get_error(tls, result));
        ERR_print_errors_fp(stderr);
        SSL_free(tls); SSL_CTX_free(context); close(fd);
        OPENSSL_cleanse(psk, sizeof(psk));
        return 1;
    }
    printf("TLS version: %s\nCipher: %s\n", SSL_get_version(tls), SSL_get_cipher(tls));

    /* Plain LuxPower frame: ReadHold (0x03), registers 0..39, with Modbus CRC. */
    unsigned char frame[38] = {0xa1, 0x1a, 1, 0, 32, 0, 1, 0xc2};
    memcpy(frame + 8, argv[2], 10);
    frame[18] = 18;
    frame[21] = 3;
    memcpy(frame + 22, argv[3], 10);
    frame[34] = 40;
    uint16_t crc = crc16(frame + 20, 16);
    frame[36] = crc & 0xff;
    frame[37] = crc >> 8;
    size_t written = 0;
    while (written < sizeof(frame)) {
        result = SSL_write(tls, frame + written, sizeof(frame) - written);
        if (result <= 0) break;
        written += (size_t)result;
    }
    int status = 1;
    if (written == sizeof(frame)) {
        unsigned char reply[65541];
        size_t received = 0, expected = 6;
        while (received < expected) {
            result = SSL_read(tls, reply + received, (int)(expected - received));
            if (result <= 0) break;
            received += (size_t)result;
            if (received >= 6 && expected == 6) {
                if (reply[0] != 0xa1 || reply[1] != 0x1a) break;
                expected = 6 + reply[4] + ((size_t)reply[5] << 8);
            }
        }
        printf("Response bytes: %zu\nResponse hex: ", received);
        for (size_t i = 0; i < received; i++) printf("%02x", reply[i]);
        printf("\n");
        if (received == expected && received >= 117 &&
            reply[0] == 0xa1 && reply[1] == 0x1a && reply[7] == 0xc2 &&
            memcmp(reply + 8, argv[2], 10) == 0 && reply[21] == 3 &&
            memcmp(reply + 22, argv[3], 10) == 0 &&
            reply[32] == 0 && reply[33] == 0 &&
            crc16(reply + 20, received - 22) ==
                (uint16_t)(reply[received - 2] | (reply[received - 1] << 8))) {
            printf("Validation: expected holding-register response, identities and CRC valid\n");
            status = 0;
        } else {
            fprintf(stderr, "No matching complete holding-register response received\n");
        }
    } else {
        fprintf(stderr, "Read request send failed\n");
    }
    SSL_free(tls);
    SSL_CTX_free(context);
    close(fd);
    OPENSSL_cleanse(psk, sizeof(psk));
    return status;
}
