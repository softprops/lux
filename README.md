# lux [![Build Status](https://travis-ci.org/softprops/lux.svg?branch=master)](https://travis-ci.org/softprops/lux) [![Coverage Status](https://coveralls.io/repos/github/softprops/lux/badge.svg?branch=master)](https://coveralls.io/github/softprops/lux?branch=master)

a kubernetes log multiplexor

## usage

```
USAGE:
    lux [FLAGS] [OPTIONS]

FLAGS:
    -f, --follow        Follows the logs as they are available
    -h, --help          Prints help information
    -t, --timestamps    Prints record timestamps
    -V, --version       Prints version information

OPTIONS:
    -l, --label <LABEL>                     Limits record to those that match a selector filter
    -n, --namespace <NAMESPACE>             Limits records to those from pods under a target namespace
        --since <SECONDS>                   Prints records since this a given number of seconds. Only one of since-time / since may be
                                            used.
        --since-time <RFC3339_TIMESTAMP>    Prints records since the given timestamp Only one of since-time / since may be used.
        --tail <N>                          Limits number of recent log records to display
```

Doug Tangren (softprops) 2016
