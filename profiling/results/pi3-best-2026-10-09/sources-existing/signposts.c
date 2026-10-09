// Public os_signpost macros keep Instruments' measurement interval separate
// from startup, warmup, and JSON output. Other platforms don't compile this file.
#include <os/signpost.h>
#include <time.h>

static os_log_t log_handle;
static os_signpost_id_t interval;

void repro_measure_init(void) {
    log_handle = os_log_create("org.femtovg.repro", OS_LOG_CATEGORY_POINTS_OF_INTEREST);
    // An enabled log is not proof that Instruments has subscribed at launch.
    // Prime discovery and allow its startup handoff before setup and warmup.
    os_signpost_event_emit(log_handle, OS_SIGNPOST_ID_EXCLUSIVE, "Profiling ready");
    const struct timespec settle = {1, 0};
    nanosleep(&settle, NULL);
}

int repro_measure_begin(void) {
    // Logging configuration can arrive after launch. Keep readiness outside
    // the measured interval and reject disabled logging instead of losing Begin.
    const struct timespec pause = {0, 10000000};
    for (unsigned attempt = 0; !os_signpost_enabled(log_handle); ++attempt) {
        if (attempt == 500) return 0;
        nanosleep(&pause, NULL);
    }
    interval = os_signpost_id_generate(log_handle);
    os_signpost_interval_begin(log_handle, interval, "Measured frames");
    return 1;
}

void repro_measure_end(void) {
    os_signpost_interval_end(log_handle, interval, "Measured frames");
}
