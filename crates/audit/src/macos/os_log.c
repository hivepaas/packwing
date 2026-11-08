// The unified log's os_log is a macro: its format has to be a literal, which
// the log keeps apart from the message, so it is written here, once. A record
// is public: it names no secret (docs/audit.md), and private, the log shows
// <private> in its place.
#include <os/log.h>
#include <stdint.h>

void packwing_os_log(os_log_t log, uint8_t type, const char *message) {
    os_log_with_type(log, (os_log_type_t)type, "%{public}s", message);
}
