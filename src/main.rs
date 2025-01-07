#![feature(portable_simd)]

mod commons;
mod easymap;
mod simdmap;

fn main() {
    let mut easymap_instance = easymap::EasyTable::new(10);
    easymap_instance.insert("hello", "world");
    easymap_instance.insert("hello2", "world2");


    println!("{:?}", easymap_instance.get(&"hello"));
    println!("{:?}", easymap_instance.get(&"hello2"));
    println!("{:?}", easymap_instance.get(&"bye"));

    let mut simdmap_instance = simdmap::SimdTable::new(10);
    for i in 0..5{
        simdmap_instance.insert(format!("hello{}", i), format!("world{}", i));
    }
    for i in 0..5{
        println!("{:?}", simdmap_instance.get(&format!("hello{}", i)));
    }
    for i in 0..5{
        simdmap_instance.insert(format!("hello{}", i), format!("world{} bye", i));
    }
    for i in 0..5{
        println!("{:?}", simdmap_instance.get(&format!("hello{}", i)));
    }

}
