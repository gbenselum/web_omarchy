#include <security/pam_appl.h>
#include <security/pam_misc.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    const char *password;
} pam_auth_data_t;

int custom_conv(int num_msg, const struct pam_message **msg,
                struct pam_response **resp, void *appdata_ptr) {
    pam_auth_data_t *data = (pam_auth_data_t *)appdata_ptr;
    struct pam_response *replies = calloc(num_msg, sizeof(struct pam_response));
    if (!replies) return PAM_CONV_ERR;

    for (int i = 0; i < num_msg; i++) {
        switch (msg[i]->msg_style) {
            case PAM_PROMPT_ECHO_OFF:
                // Password prompt - use our password
                replies[i].resp = strdup(data->password);
                replies[i].resp_retcode = 0;
                break;
            case PAM_PROMPT_ECHO_ON:
                // Username prompt - should not happen as we set PAM_USER
                replies[i].resp = strdup("");
                replies[i].resp_retcode = 0;
                break;
            case PAM_ERROR_MSG:
            case PAM_TEXT_INFO:
                // Just acknowledge
                replies[i].resp = strdup("");
                replies[i].resp_retcode = 0;
                break;
            default:
                free(replies);
                return PAM_CONV_ERR;
        }
    }
    *resp = replies;
    return PAM_SUCCESS;
}

int main(int argc, char *argv[]) {
    if (argc != 3) {
        fprintf(stderr, "Usage: %s <username> <password>\n", argv[0]);
        return 1;
    }

    const char *username = argv[1];
    const char *password = argv[2];

    pam_auth_data_t auth_data = { .password = password };

    struct pam_conv conv = {
        custom_conv,
        &auth_data
    };

    pam_handle_t *pamh = NULL;
    int retval = pam_start("system-auth", username, &conv, &pamh);
    if (retval != PAM_SUCCESS) {
        fprintf(stderr, "pam_start failed: %s\n", pam_strerror(pamh, retval));
        return 1;
    }

    // Set credentials
    pam_set_item(pamh, PAM_USER, username);

    // Authenticate
    retval = pam_authenticate(pamh, 0);
    if (retval != PAM_SUCCESS) {
        fprintf(stderr, "Authentication failed: %s\n", pam_strerror(pamh, retval));
        pam_end(pamh, retval);
        return 1;
    }

    // Check account validity
    retval = pam_acct_mgmt(pamh, 0);
    if (retval != PAM_SUCCESS) {
        fprintf(stderr, "Account management failed: %s\n", pam_strerror(pamh, retval));
        pam_end(pamh, retval);
        return 1;
    }

    pam_end(pamh, PAM_SUCCESS);
    return 0;
}