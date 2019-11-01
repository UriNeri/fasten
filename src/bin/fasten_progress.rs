extern crate getopts;
extern crate fasten;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::env;

use fasten::fasten_base_options;

fn main(){
    let args: Vec<String> = env::args().collect();
    let mut opts = fasten_base_options();
    // Options specific to this script
    //opts.optopt("","min-length","Minimum read length allowed","INT");
    //opts.optopt("","min-quality","Minimum quality allowed","FLOAT");
    opts.optopt("","id","Progress identifier","STRING");
    opts.optopt("","update-every","Update progress every n reads.","INT");
    //opts.optflag("","print-reads","Print the reads as they are being validated (useful for unix pipes)");
    //opts.optflag("v","verbose","");

    let matches = opts.parse(&args[1..]).expect("ERROR: could not parse parameters");
    if matches.opt_present("help") {
        println!("Prints a progress meter for number of fastq entries.\n{}", opts.usage(&opts.short_usage(&args[0])));
        std::process::exit(0);
    }

    let my_file = File::open("/dev/stdin").expect("Could not open file");
    let my_buffer=BufReader::new(my_file);

    let lines_per_read :usize = 4;
    let update_every :usize  = {
        if matches.opt_present("update-every") {
            let tmp :usize = 
                matches.opt_str("update-every")
                 .expect("ERROR parsing update-every")
                 .parse()
                 .expect("ERROR parsing update-every as INT");
            tmp * lines_per_read
            //tmp
        } else {
            100
        }
    };

    let mut line_counter = 0;
    for _line in my_buffer.lines() {
        //let _line=line.expect("ERROR: did not get a line");
        line_counter += 1;

        // TODO pattern match for each kind of line:
        // id, seq, +, qual
        match line_counter % update_every {
            0=>{
                //println!("UPDATE: {}", line_counter);
                eprint!("\r{}", line_counter/lines_per_read);
                //eprint!(".");
            }
            _=>{

            }
        }
    }
    eprint!("\n");

    if matches.opt_present("verbose") {
        let msg = format!("Finished progress on {} reads", line_counter);
        fasten::logmsg(&msg);
    }
}


