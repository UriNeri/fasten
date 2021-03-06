extern crate getopts;
extern crate fasten;
extern crate multiqueue;

use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;

use fasten::fasten_base_options;

use std::env;

fn main(){
    let args: Vec<String> = env::args().collect();
    let mut opts = fasten_base_options();

    // script-specific options
    opts.optopt("","min-length","Minimum length for each read in bp","INT");
    opts.optopt("","min-avg-quality","Minimum average quality for each read","FLOAT");
    opts.optopt("","min-trim-quality","Trim the edges of each read until a nucleotide of at least X quality is found","INT");

    let matches = opts.parse(&args[1..]).expect("ERROR: could not parse parameters");

    if matches.opt_present("h") {
        println!("{}", opts.usage(&opts.short_usage(&args[0])));
        std::process::exit(0);
    }

    let mut min_length :usize = 0;
    if matches.opt_present("min-length") {
        min_length = matches.opt_str("min-length")
            .expect("ERROR: could not read the minimum length parameter")
            .parse()
            .expect("ERROR: min-length is not an integer");
    }

    let mut min_avg_qual :f32 = 0.0;
    if matches.opt_present("min-avg-quality") {
        min_avg_qual = matches.opt_str("min-avg-quality")
            .expect("ERROR: could not read the minimum average quality parameter")
            .parse()
            .expect("ERROR: min-avg-qual is not an integer");
    }

    let mut min_trim_qual :u8 = 0;
    if matches.opt_present("min-trim-quality") {
        min_trim_qual = matches.opt_str("min-trim-quality")
            .expect("ERROR: could not read the minimum trim quality parameter")
            .parse()
            .expect("ERROR: min-trim-qual is not an integer");
    }

    let lines_per_read={
        if matches.opt_present("paired-end") {
            8
        }else{
            4
        }
    };


    // Read the file and send seqs to threads
    let my_file = File::open("/dev/stdin").expect("Could not open file");
    let my_buffer=BufReader::new(my_file);
    
    let mut id1   :String = String::new();
    let mut id2   :String = String::new();
    let mut seq1  :String = String::new();
    let mut seq2  :String = String::new();
    let mut qual1 :String;
    let mut qual2 :String;
    let mut seq1_trimmed  = String::new();
    let mut seq2_trimmed :String;
    let mut qual1_trimmed = String::new();
    let mut qual2_trimmed :String;

    for (i,wrapped_line) in my_buffer.lines().enumerate() {
        let line = wrapped_line.expect("ERROR: could not read line");
        match i % lines_per_read {
            // read ID
            0=>{
                id1 = line;
            }
            4=>{
                id2 = line;
            }

            // Sequence
            1=>{
                seq1 = line;
            }
            5=>{
                seq2 = line;
            }

            // Qual line. If we've gotten here, then we can also trim/filter/print
            // First qual line
            3=>{
                qual1 = line;

                // Trim
                let tuple=trim(&seq1,&qual1,min_trim_qual);
                seq1_trimmed=tuple.0;
                qual1_trimmed=tuple.1;

                // If this is single end, go ahead and filter/print
                if lines_per_read==4 {
                    if seq1_trimmed.len() >= min_length && avg_quality(&qual1_trimmed) >= min_avg_qual {
                        println!("{}\n{}\n+\n{}",
                             id1,seq1_trimmed,qual1_trimmed,
                             );
                    }
                }

            }
            // Second qual line
            7=>{
                qual2 = line;

                // Trim
                let tuple=trim(&seq2,&qual2,min_trim_qual);
                seq2_trimmed=tuple.0;
                qual2_trimmed=tuple.1;

                // Since we are at the second qual line, this is PE and we can
                // go ahead with filter/print and not check for the PE param.

                if seq1_trimmed.len() >= min_length && seq2_trimmed.len() >= min_length 
                    && avg_quality(&qual1_trimmed) >= min_avg_qual 
                    && avg_quality(&qual2_trimmed) >= min_avg_qual {

                    println!("{}\n{}\n+\n{}\n{}\n{}\n+\n{}",
                         id1,seq1_trimmed,qual1_trimmed,
                         id2,seq2_trimmed,qual2_trimmed
                         );
                }
            }
            _=>{}
        }

    }

}

/// determine average quality of a qual cigar string
fn avg_quality(qual: &String) -> f32 {
    let mut total :u32 = 0;
    for qual_char in qual.chars() {
        total += qual_char as u8 as u32;
    }
    let avg = (total as f32 / qual.len() as f32) - 33.0;
    return avg;
}

/// Trim the ends of reads with low quality
fn trim(seq: &String, qual: &String, min_qual: u8) -> (String,String) {
    let mut trim5 :usize=0;
    let mut trim3 :usize=qual.len();

    let offset_min_qual = min_qual + 33;
    
    // 5'
    for qual in qual.chars(){
        if (qual as u8) < offset_min_qual {
            trim5+=1;
        } else {
            break;
        }
    }

    // 3'
    for qual in qual.chars().rev() {
        if (qual as u8) < offset_min_qual {
            trim3-=1;
        } else {
            break;
        }
    }

    let new_seq :String;
    let new_qual:String;
    
    if trim5 >= trim3 {
        new_seq = String::new();
        new_qual= String::new();
    } else {
        new_seq  =  seq[trim5..trim3].to_string();
        new_qual = qual[trim5..trim3].to_string();
    }
    return(new_seq,new_qual);
}

