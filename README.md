# mix-emulator

Emulator of MIX written in Rust

## How to use

```sh
$ mix-emulator --help
mix-emulator 0.1.0

USAGE:
    mix-emulator [FLAGS] [OPTIONS] <input file>

FLAGS:
    -c, --compile       compiler mode
    -f, --format        formatter mode
    -h, --help          Prints help information
    -i, --inplace       inplace edit, if specified
    -S, --preprocess    preprocessor mode
    -V, --version       Prints version information
    -v, --verbose       

OPTIONS:
        --cardpunch <card-punch>        
        --cardreader <card-reader>      
        --disk0 <disk0>                 
        --disk1 <disk1>                 
        --disk2 <disk2>                 
        --disk3 <disk3>                 
        --disk4 <disk4>                 
        --disk5 <disk5>                 
        --disk6 <disk6>                 
        --disk7 <disk7>                 
        --lineprinter <line-printer>    
        --papertape <paper-tape>        
        --tape0 <tape0>                 
        --tape1 <tape1>                 
        --tape2 <tape2>                 
        --tape3 <tape3>                 
        --tape4 <tape4>                 
        --tape5 <tape5>                 
        --tape6 <tape6>                 
        --tape7 <tape7>                 
        --twt <type-writer-terminal>    

ARGS:
    <input file>
```

### Examples

```sh
$ mix-emulator -v taocp/prime500.mix
$ mix-emulator -v --cardreader taocp/swap1-input.txt taocp/swap1.mix
$ mix-emulator -v --cardreader taocp/swap2-input.txt taocp/swap2.mix
$ mix-emulator -v --tape1 taocp/topsort-input.txt taocp/topsort.mix
```

If you want to profile the code, use `-v` option.
