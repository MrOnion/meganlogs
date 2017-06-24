
# MeganLogs

Dead simple terminal logger for Megasquirt(tm) ECU:s. Just ride-n-log with no extra hassle.

### Features/Limitations/Annoyances:
  * Serial request-response protocol only, no CAN (means: no high speed logging)
  * Supports only 2nd gen CRC based Megasquirt serial protocol (aka 'newserial')
  * FRD binary output format only (use TunerStudioMS to convert to MSL)
  * Curses UI, so used/tested only on Linux/OSX (no Windows support)
  * Ugly sleep based data rate limiter (5-25 reads / sec)
  * Simple and easy to use key bindings (helpful when riding solo)
  * Names logfiles with given prefix, current date/time, session ID and run ID (ex. mylog_20161002_121326_s1_run1.frd)
    * Session ID increments when new one is created.
    * Run ID increments on every logging toggle

### Key bindings
  - `<space>`: toggle logging on/off
  - `<enter>`: create a marker on a running logfile
  - `s`: create a new session (logging needs to be off)

