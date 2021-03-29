#!/bin/bash

# In case of a bug in the compiler, use comby-reducer to cut of any code,
# which does not contribute to the problem.

exec comby-reducer src/main.rs.backup.rs --file /tmp/tmp.rs --lang .rs -- ./sbuild.sh @@
