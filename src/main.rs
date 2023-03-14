
use image::Rgb;
use std::{io, thread};
use std::sync::mpsc::{self, Sender};

fn main(){

    let mut g:mainthread = mainthread{x:0.0, y:0.0, xsize:1000, ysize:1000, zoom:1.0, numthreads:10};
    
    loop{
        
        g.generate();
        println!("Awaiting Move:");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("unable to read user input");

        match input.as_str(){
            "s\n" => {g.y = g.y + g.zoom/4.0},
            "w\n" => {g.y = g.y-g.zoom/4.0},
            "a\n" => {g.x = g.x-g.zoom/4.0},
            "d\n" => {g.x = g.x+g.zoom/4.0},
            "z\n" => {g.zoom = g.zoom/2.0},
            "x\n" => {g.zoom = g.zoom*2.0},
            "q\n" => panic!("interupt"),
            _ => {print!("Unable to read input. ");
                 continue;}
        }
    }
}
#[derive(Copy, Clone)]
struct mainthread{x:f64, y:f64, xsize:u32, ysize:u32, zoom:f64, numthreads:u8}
impl mainthread {
    fn generate(self){

        let exe_path = std::env::current_exe().unwrap().parent().unwrap().join("grad.tiff");
        let sample_image = image::open(exe_path).unwrap().to_rgb8();
        let mut image = image::RgbImage::new(self.xsize, self.ysize);
        let mut threads = vec![];
        let mut channels:Vec<mpsc::Receiver<(u32, u32, u32)>> = vec![];
        let x_change:f64 = self.zoom/self.xsize as f64;
        let y_change:f64 = self.zoom/self.ysize as f64;
        
        //set up the amount of columns we should assign to each individual thread
        let mut columns_per_thread:Vec<u32> = vec![];
        //if there is no remainder we can equally divide the columns amoung the threads

        if self.numthreads == 0 {
            panic!("No threads being used")
        } else if self.xsize % self.numthreads as u32 == 0{
            for _i in 0..self.numthreads{
                columns_per_thread.push(self.xsize/self.numthreads as u32);
            }
            //if there is a remainder, set aside one thread to handle it and equally divide the new amout of columns amoung all other threads
        } else {
            for _i in 0..self.numthreads{
                columns_per_thread.push((self.xsize - (self.xsize % self.numthreads as u32-1)) / self.numthreads as u32-1);
            }
            columns_per_thread.push(self.xsize % self.numthreads as u32-1);
        }
        
        let mut image_pointer_x:Vec<Vec<u32>> = vec![];
        //image_pointer_y is just equal to 0 for all threads 
        let mut image_pointer_x_counter = 0;

        //starting point on fractal for threads 
        let mut fractal_pointer_x:Vec<Vec<f64>> = vec![];
        let mut fractal_pointer_y:Vec<Vec<f64>> = vec![];

        let mut fractal_pointer_x_counter:f64 = self.x - (x_change * self.xsize as f64/2.0);
        let fractal_pointer_y_counter:f64 = self.y - (y_change * self.ysize as f64/2.0);
        //assign parameter vectors to threads 
        for i in 0..self.numthreads{
            let mut image_pointer_to_push:Vec<u32> = vec![];
            let mut fractal_pointer_x_to_push:Vec<f64> = vec![];
            let mut fractal_pointer_y_to_push:Vec<f64> = vec![];

            for _i2 in 0..columns_per_thread[i as usize]{

                image_pointer_to_push.push(image_pointer_x_counter);
                fractal_pointer_x_to_push.push(fractal_pointer_x_counter);
                fractal_pointer_y_to_push.push(fractal_pointer_y_counter);
                image_pointer_x_counter += 1;
                fractal_pointer_x_counter += x_change;
            }
            fractal_pointer_x.push(fractal_pointer_x_to_push);
            fractal_pointer_y.push(fractal_pointer_y_to_push);
            image_pointer_x.push(image_pointer_to_push);
        }
        
        for i in 0..self.numthreads{
            let (tx, rx) = mpsc::channel();
            let fractal_pointer_x = fractal_pointer_x[i as usize].clone();
            let fractal_pointer_y = fractal_pointer_y[i as usize].clone();
            let image_pointer_x = image_pointer_x[i as usize].clone();

            channels.push(rx);
            threads.push(thread::spawn(move || {
                thread_target(&fractal_pointer_x, &fractal_pointer_y, &image_pointer_x, &self.ysize, &y_change, &tx);
            }))
        }


    
        loop{

            if channels.len() == 0{
                break;
            }
            for i in 0..channels.len(){
                let (x, y, p) = channels[i as usize].recv().unwrap();

                if x == 0 && y == 0 && p == 0 {
                    //handle thread exit 
                    threads.remove(i as usize);
                    channels.remove(i as usize);
                    break;
                }
                if p == 255{
                    image.put_pixel(x, y, Rgb([0, 0, 0]));
                } else {
                image.put_pixel(x, y, *sample_image.get_pixel(p as u32, 0));
                }
                    
        

            }
        }

    let exe_path = std::env::current_exe().unwrap().parent().unwrap().join("fractal_image.tiff");
    image.save(exe_path);


    }
}
 

fn thread_target(xpoint:&Vec<f64>, ypoint:&Vec<f64>, imagexpoint:&Vec<u32>, repetitions:&u32, ychange:&f64, tx:&Sender<(u32, u32, u32)>) {

    for i in 0..xpoint.len(){

        let mut y:f64 = ypoint[i as usize];
        let x:f64 = xpoint[i as usize];
        let imx:u32 = imagexpoint[i as usize];

        for r in 0..*repetitions{

            let pix:u32 = test_pixel(x, y, 500 as u32);
            tx.send((imx as u32, r as u32, pix as u32));
            
            y += ychange;
        }
    }
    //this signals to the main thread that all the data has been sent 
    tx.send((0, 0, 0));
}

//struct for working with complex numbers
//can add another inum to self, and square itself
//TODO: add general multiplication for other sets


struct inum{r:f64, i:f64}

impl inum{
    fn add (&mut self, num:inum){
        self.r = self.r + num.r;
        self.i = self.i + num.i;
    }

    fn square (&mut self){

        let r:f64 = self.r*self.r - self.i*self.i;
        let i:f64 = self.r * self.i * 2.0;

        self.r = r;
        self.i = i;

    }
}

//Tests mandelbrot set coord 
fn test_pixel(x:f64, y:f64, rep:u32) -> u32 {
    let mut i:inum = inum { r:0.0, i: 0.0  };

    for item in 1..rep{
        i.square();
        i.add(inum { r: x, i: y });

        if (i.r*i.r)+(i.i*i.i) > 25.0 {
            return item % 299;
        }
    }
    return 255;
}


