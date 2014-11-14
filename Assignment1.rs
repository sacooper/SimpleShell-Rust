use std::io;
use std::io::process::Command;
use std::io::process;

struct JobEntry{
	process : process::Process,
	name : String
}

#[deriving(Clone)]
struct HistoryEntry{
	args : Vec<String>,
	background : bool
}


fn main(){
	let mut jobs : Vec<JobEntry> = Vec::new();
	let mut history : Vec<HistoryEntry> = Vec::new();

	fn execute(args : &Vec<&str>, background : bool, history : &mut Vec<HistoryEntry>, jobs : &mut Vec<JobEntry>) -> Result<(), ()> {
		assert!(args.len() > 0);
		match args[0] {
			"exit" => {Err(())}
			"history" =>{
				if history.is_empty(){
					println!("Nothing entered yet");
				} else{
					let y : Vec<&HistoryEntry> = history.iter().rev().take(10u).collect();
					for (i, x) in  y.iter().enumerate().rev(){
						let name = x.args.iter().fold(String::new(), |acc, x|{acc.add(x).add(&" ")});
						let name = if x.background {name} else {name.add(&"&")};
						println!("{}: {}", history.len() - i, name)
					}
				};
				Ok(())}
			"r" => {
				if args.len() == 1 {
					match history.clone().last(){
						Some(entry) => {
							let args = entry.args.iter().map(|x|{x.as_slice()}).collect();
							execute(&args, entry.background, history, jobs)}
						None => {println!("Nothing entered yet"); Ok(())}}
				} else {
					let mut found = false;
					let mut res = Ok(());
					for x in history.clone().iter().rev() {
						if x.args[0].as_slice().char_at(0) == args[1].char_at(0){
							found = true;
							res = execute(&(x.args.iter().map(|x|{x.as_slice()}).collect()), x.background, history, jobs);
							break;
						}
					}
					if !found {println!("No arguments starting with {}", args[1]); Ok(())} else{res}
				}
			}
			"jobs" => {
				if jobs.is_empty(){println!("No jobs currently running")}
				else{
					for (i, x) in jobs.iter().enumerate(){
						println!("[{}] {}", i+1, x.name)}}
				Ok(())
			}
			"fg" => {
				if args.len() > 1 {
					let x : Option<uint> = from_str(args[1]);
					match x {
						Some(x) =>{
							if x > jobs.len() || x <= 0 {println!("No job [{}]", x)}
							else{
								let _ = jobs[x-1].process.wait();
								jobs.remove(x-1);
							};
							Ok(())
						}
						None => {println!("Not a valid number"); Ok(())}
					}
				} else if jobs.len() == 1 {
					let _ = jobs[0].process.wait();
					jobs.remove(0);
					Ok(())
				}else {
					println!("No number specified for 'fg'"); Ok(())
				}
			}
			"kill" => {
				if args.len() > 1 {
					let x : Option<uint> = from_str(args[1]);
					match x {
						Some(x) =>{
							if x > jobs.len() || x <= 0{println!("No job [{}]", x)}
							else{
								let _ = jobs[x-1].process.signal_exit();
								jobs.remove(x-1);
							};
							Ok(())
						}
						None => {println!("Not a valid number"); Ok(())}
					}
				} else if jobs.len() == 1 {
					let _ = jobs[0].process.signal_exit();
					jobs.remove(0);
					Ok(())
				}else {
					println!("No number specified for 'kill'"); Ok(())
				}
			}
			_ => {
				let mut prog = Command::new(args[0]);
				prog.args(args.tail().as_slice());
				prog.stdin(process::InheritFd(0));
				prog.stdout(process::InheritFd(1));
				prog.stderr(process::InheritFd(2));


				if background {
					let (tx, rx): (Sender<process::Process>, Receiver<process::Process>) = channel();
					spawn(proc() {
						match prog.spawn() {
							Ok(p) => tx.send(p),
							Err(e) => println!("Invalid command: {}", e.kind)}});
					let p = rx.recv();
					jobs.push(JobEntry{process : p, name : args.iter().fold(String::new(), |acc, x|{acc.add(x).add(&" ")})})}
				else {
					match prog.spawn(){
						Ok(_) => (),
						Err(e) => println!("Invalid command: {}", e.kind)
					}}

				history.push(HistoryEntry{args : args.iter().map(|x|{String::from_str(*x)}).collect(), background : background});


				Ok(())
			}
		}
	};

	fn setup<'r>(input : &'r String) -> (Vec<&'r str>, bool){
		let mut ret = Vec::new();
		let mut background = false;
		for c in input.as_slice().split(' '){
			match c {
			"&" => {background = true},
			_ => ret.push(c)}}
		(ret, background)}



	loop{
		print!("COMMAND -> ");
		let input : String = io::stdin().read_line().unwrap_or(String::new());
		// If they just entered a newline, continue, if its empty, means ^-D, so exit
		if input.is_empty() {break}
		else if input == String::from_str("\n"){continue};
		let input : String = input.replace("\n", "");// Remove trailing newline
		let (setup_res, background) = setup(&input);
		if execute(&setup_res, background, &mut history, &mut jobs).is_err(){break}
	}

	for x in jobs.iter(){
		let _ = process::Process::kill(x.process.id(), process::PleaseExitSignal);}

	println!("exit")

}
