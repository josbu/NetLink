![](https://img.shields.io/github/downloads/rustp2p/NetLink/total?logo=github&label=Download)
[![Apache-2.0](https://img.shields.io/github/license/rustp2p/NetLink?style=flat)](https://github.com/rustp2p/NetLink/blob/master/LICENSE)

<p align="center">
  <a href="./README.zh-CN.md">简体中文</a> |
  <a href="./README.md">English</a>
</p>

`NetLink` 是建立在[rustp2p](https://crates.io/crates/rustp2p)库基础上的去中心化的网络工具.

## 使用

```
管理员权限直接运行: ./netLink.exe 
管理员权限设置http服务运行: ./netLink.exe --api_addr 192.168.0.1:8080
管理员权限命令行输入: netLink.exe [OPTIONS] --local <LOCAL IP> --group-code <GROUP CODE>

Commands:
  cmd   Backend command
  help  打印帮助信息

Options:
  -p, --peer <PEER>              远端节点地址(需可直接访问). 如: -p tcp://192.168.10.13:23333 或 -p udp://192.168.10.23:23333 或 -p txt://域名
  -l, --local <LOCAL IP>         设定本地节点地址 CIDR格式. 如: -l 10.26.0.2/24 不填掩码则不监听虚拟网段，相当于只能当中继节点
  -g, --group-code <GROUP CODE>  节点所在组的名称(最大长度16),只有同一组的节点才能进行数据访问
  -P, --port <PORT>              本地监听地址
  -b, --bind-dev <DEVICE NAME>   指定流量出口网卡名. 如: -b eth0
      --threads <THREADS>        设置使用线程数, 默认两个线程
  -e, --encrypt <PASSWORD>       设定秘钥并开启加密. 如: -e "password"
  -a, --algorithm <ALGORITHM>    设定加密算法. 可选择算法: aes-gcm/chacha20-poly1305/xor, 默认是：chacha20-poly1305
      --exit-node <EXIT_NODE>    网关节点，请配合'--bind-dev'使用
      --tun-name <TUN_NAME>      设定本地tun的名称
      --mtu <MTU>                设置mtu
  -X, --filter <FILTER>          Group code 白名单，使用正则表达式
  -f, --config <CONFIG>          使用配置文件启动
      --api-addr <API_ADDR>      设置http服务地址，默认是：127.0.0.1:23336
      --api-disable              禁用http服务
      --username <USER_NAME>     设置http登录账号
      --password <PASSWORD>      设置http登录密码
  -h, --help                     帮助
  -V, --version                  版本

 ```

## 使用配置文件启动

<details> <summary>展开</summary>

```yaml
## ./netLink --config <config_file_path>
## 按需修改

## 后台api服务监听地址. 默认值 "127.0.0.1:23336"
#api_addr: "127.0.0.1:23336"
## 不使用api服务，则设置 api_disable:true
#api_disable: false
## 工作线程数. 默认值 2
#threads: 2
## 登录用户名
#username: 
## 登录密码
#password: 
## 组编号，必填
group_code: String
## 虚拟ipv4 必填
node_ipv4: "10.26.1.2"
## 网段，默认值24，填0则不监听tun网络，此时只能当中继节点
#prefix: 24
## 虚拟ipv6,会自动生成
# node_ipv6: 
# prefix_v6: 96

## tun网卡名称，会自己生成
#tun_name: "tun3"
## 开启加密，设置加密密码
#encrypt: "password"
## 加密算法. 可选 aes-gcm/chacha20-poly1305/xor. 默认值 chacha20-poly1305
#algorithm: "chacha20-poly1305"
##监听端口. 默认值 23333
# port: 23333
## 对端地址
#peer:
#   - udp://192.168.10.23:23333
#   - tcp://192.168.10.23:23333
## 使用网卡名称绑定出口网卡
#bind_dev_name: "eth0"
## 全局出口，配合 "bind_dev_name"一起使用
#exit_node: 
## 设置网卡mtu
#mtu: 1400
## Group code 白名单，使用正则表达式
#group_code_filter:
#   - ^test # 放行test开头的
#   - ^pass$ # 全匹配pass

## tun服务 区分udp和tcp服务
#udp_stun:
#   - stun1.l.google.com:19302
#   - stun2.l.google.com:19302
#tcp_stun:
#   - stun.flashdance.cx
#   - stun.nextcloud.com:443

```

</details>

## Web UI

[netlink-app](https://github.com/rustp2p/netlink-app)

### 使用方法：

#### 一. 使用浏览器启动：

1. 命令行启动netlink
2. 在浏览器使用[http://127.0.0.1:23336](http://127.0.0.1:23336)访问

#### 二. tauri可执行文件启动：

1. 命令行启动netlink
2. 打开netlink-app

## 特性

| Features                |   |
|-------------------------|---| 
| **去中心化**                | ✅ |
| **跨平台**                 | ✅ |
| **NAT穿透**               | ✅ | 
| **子网路由**                | ✅ | 
| **加密**                  | ✅ | 
| **高性能**                 | ✅ | 
| **HTTP/Rust/C/JNI API** | ✅ | 
| **Ipv4/Ipv6**           | ✅ | 
| **UDP/TCP**             | ✅ | 

## 快速上手

```mermaid
flowchart LR
    subgraph Node-A 8.210.54.141
        node_a[10.26.1.2/24]
    end
    subgraph Node-B
        node_b[10.26.1.3/24]
    end

    subgraph Node-C
        node_c[10.26.1.4/24]
    end

    node_a <-----> node_b
    node_c <-----> node_b
    node_a <-----> node_c
```

1. Node-A
    ```
    ./netLink --group-code 123 --local 10.26.1.2/24
    ```
2. Node-B
    ```
    ./netLink --group-code 123 --local 10.26.1.3/24 --peer 8.210.54.141:23333
    ```
3. Node-C
    ```
    ./netLink --group-code 123 --local 10.26.1.4/24 --peer 8.210.54.141:23333
    ```
4. 节点 A, B, and C 可以互相访问

## 多节点

```mermaid
flowchart LR
    subgraph Node-A 8.210.54.141
        node_a[10.26.1.2/24]
    end
    subgraph Node-B
        node_b[10.26.1.3/24]
    end

    subgraph Node-C 192.168.1.2
        node_c[10.26.1.4/24]
    end
    subgraph Node-D
        node_d[10.26.1.5/24]
    end
    node_b -----> node_a
    node_c -----> node_a
    node_d -----> node_c
```

```
Node-A: ./netLink --group-code 123 --local 10.26.1.2/24
Node-B: ./netLink --group-code 123 --local 10.26.1.3/24 --peer 8.210.54.141:23333
Node-C: ./netLink --group-code 123 --local 10.26.1.4/24 --peer 8.210.54.141:23333
Node-D: ./netLink --group-code 123 --local 10.26.1.5/24 --peer 192.168.1.2:23333
```

所有已连接的节点可以互相访问

多节点也可以通过'-peer'连接.  
example：

```
Node-A: ./netLink --group-code 123 --local 10.26.1.2/24
Node-B: ./netLink --group-code 123 --local 10.26.1.3/24 --peer 8.210.54.141:23333
Node-C: ./netLink --group-code 123 --local 10.26.1.4/24 --peer 8.210.54.141:23333
Node-D: ./netLink --group-code 123 --local 10.26.1.5/24 --peer 192.168.1.2:23333 --peer 8.210.54.141:23333
```

此外，--peer也支持DNS TXT记录重定向，用法：--peer txt://域名，然后在对应域名添加TXT记录，例如 tcp://8.210.54.141:23333


## 子网路由

```
Public Node-S: 8.210.54.141

Subnet 1: 192.168.10.0/24
      Node-A: 192.168.10.2
      Node-B: 192.168.10.3
      
Other subnet:   
      Node-C

Node-S: ./netLink --group-code xxxx --local 10.26.1.1
Node-A: ./netLink --group-code 123 --local 10.26.1.3/24 --peer 8.210.54.141:23333
Node-C: ./netLink --group-code 123 --local 10.26.1.4/24 --peer 8.210.54.141:23333

Node-C <--> Node-A(192.168.10.2) <--> Node-B(192.168.10.3)
```

1. **第一步 : 节点Node-A配置转发网卡**

> 转发所有来源地址在网段10.26.1.0/24下的流量到指定网卡

**Linux**

   ```
   sudo sysctl -w net.ipv4.ip_forward=1
   sudo iptables -t nat -A POSTROUTING  -o eth0 -s 10.26.1.0/24 -j MASQUERADE
   ```

**Windows**

   ```
   New-NetNat -Name testSubnet -InternalIPInterfaceAddressPrefix 10.26.1.0/24
   ```

**Macos**

   ```
   sudo sysctl -w net.ipv4.ip_forward=1
   echo "nat on en0 from 10.26.1.0/24 to any -> (en0)" | sudo tee -a /etc/pf.conf
   sudo pfctl -f /etc/pf.conf -e
   ```

2. **第二步 : 节点Node-C的路由设置**

> 将目标地址在网段192.168.10.0/24下的流量通过路由代理到本地tun并且发送到网关10.26.1.3(即 节点Node-A的虚拟地址)

**Linux**

   ```
   sudo ip route add 192.168.10.0/24 via 10.26.1.3 dev <netLink_tun_name>
   ```

**Windows**

   ```
   route add 192.168.10.0 mask 255.255.255.0 10.26.1.3 if <netLink_tun_index>
   ```

**Macos**

   ```
   sudo route -n add 192.168.10.0/24 10.26.1.3 -interface <netLink_tun_name>
   ```

此时, Node-C可以通过Node-A访问Node_B, 就像Node-C和Node-B是直连的一样

## 链接库集成

https://github.com/rustp2p/NetLink_adapter

## 联系

- 电报: https://t.me/+hdMW5gWNNBphZDI1
- QQ群: 211072783

## 免费社区节点

- --peer tcp://198.46.149.74:23333
