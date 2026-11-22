# CI: known open items

No check is red on main.

Run 44989168468 (#176) passed on its second attempt. The first attempt exposed
one flaky check: `HomeCalendar.visual.spec.ts` timed out on the 2s tooltip wait
and passed when rerun; if it recurs, give that wait more time instead of
rerunning the job.

The two items this file used to track are closed:

- Windows dual-installer packaging: fixed in df13559 and b491bf7.
- Coverage job: it was never below the 59% floor (63.55% lines); the job failed
  on `command_observability_parity`, fixed in 46cdaf9.
