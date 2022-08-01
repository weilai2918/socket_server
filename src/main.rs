use std::net::SocketAddr;
use tokio::sync::broadcast::{Sender};

use tokio::net::{TcpSocket, TcpStream};
use tokio::io::{Error, AsyncReadExt, AsyncWriteExt};

async fn process(tcp_stream:&mut TcpStream,tx: &Sender<(String,SocketAddr)>)-> bool{
    let (mut read_half,_write_half) = tcp_stream.split();
    
    let mut buf = vec![0;1024];
    
    //读取消息
    match read_half.read_buf(&mut buf).await {
        
        Ok(_n) => {
            //转换字符串
            let res = String::from_utf8(buf).unwrap();
            //println!("size:{},content:{}",n,res);
            let peer_addr = tcp_stream.peer_addr().unwrap();
            //通过通道发送
            tx.send((res,peer_addr)).unwrap();
            return true;
        }
        Err(err) => {   
            println!("err : {:?}",err);
            return false;
        }
    }
}


#[tokio::main]
async fn main() -> Result<(),Error> {
    
    println!(r"
    _                                  
_ __ _   _ ___| |_   ___  ___ _ ____   _____ _ __ 
| '__| | | / __| __| / __|/ _ \ '__\ \ / / _ \ '__|
| |  | |_| \__ \ |_  \__ \  __/ |   \ V /  __/ |   
|_|   \__,_|___/\__| |___/\___|_|    \_/ \___|_|   
                                     ");

   

    let addr = "127.0.0.1:5555".parse().unwrap();

    let socket = TcpSocket::new_v4().unwrap();
    let _ = socket.bind(addr);
    let listen = socket.listen(1024).unwrap();

    println!("服务启动成功,端口:5555");

    let (tx,_rx) = tokio::sync::broadcast::channel(1024);

    loop {
        //等待客户端的连接
        let (mut tcp_stream,_) = listen.accept().await.unwrap();

        let tx = tx.clone();
        let mut rx = tx.subscribe();
        //启动线程
        tokio::spawn(async move{
            //循环处理事件
            loop {
                //只要是返回一个就接结束等待的其他线程
                tokio::select! {
                    //处理消息
                    result = process(&mut tcp_stream,&tx) => {
                        //如果出现异常结束循环
                        if !result {
                            break;
                        }
                    }
                    //发送消息
                    result = rx.recv() => {
                        let (msg,addr) = result.unwrap();
                        //判断给除了自己的客户端发送消息
                        if addr != tcp_stream.peer_addr().unwrap(){
                            let _ = tcp_stream.write_all(msg.as_bytes()).await; 
                        } 
                    }
                }
            }
            //获取客户端地址
            let ip = tcp_stream.peer_addr().unwrap();
            println!("{:?}:断开连接",ip);
        });
        
    }
}