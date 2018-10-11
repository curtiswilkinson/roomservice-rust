extern crate glob;
extern crate rayon;
extern crate sha2;
extern crate subprocess;

pub mod roomservice;
use roomservice::room::RoomBuilder;
use roomservice::RoomserviceBuilder;

fn main() {
    // use subprocess::Exec;
    // use subprocess::Redirection;
    // let mut x = Exec::shell("toh hey")
    //     .cwd("./")
    //     .stderr(Redirection::Pipe)
    //     .popen()
    //     .unwrap();

    // let res = x.wait().unwrap();

    // println!("{:?} {:?}", res, x);
    let mut roomservice = RoomserviceBuilder::new("../unity/services/".to_string());

    roomservice.add_room(RoomBuilder::new("./", None, "./**/*.ts"));
    // roomservice.add_room(Room::new("./", None, "./**/*"));
    println!("{:?}", roomservice);
    roomservice.exec()
}
