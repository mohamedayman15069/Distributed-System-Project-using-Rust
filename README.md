# Distributed-System-Project
This is a distributed system project handling the load balancing and peer-to-peer election using RUST

## Introduction

The aim of this project is to simulate the interactions between servers and clients in a distributed system. We implemented this project using the language Rust. Our system consists of two client machines and three server machines. Our project implements two main concepts: load balancing and fault tolerance. Load balancing is concerned with the distribution of client requests over the available servers at any given moment, such that the loads on the servers would be near equal. The fault tolerance mechanism is concerned with the ability of the system to continue operating in the case of a server failure. 

## Design & Implementation

### Network Protocol
For the choice of network protocols, we used UDP (User Datagram Protocol). UDP is a connectionless protocol; which means that the request packet is sent directly, without the need to establish a connection first (handshake). On the other hand, the TCP (Transmission Control Protocol) is a connection-oriented protocol, where a connection must be established before sending request packets. The advantages of using UDP over TCP include its simplicity, higher speeds, lower latency, avoiding retransmission delays and the overhead of establishing the connection.

### Clients
The clients are the end-users of the system. In our network, we have 2 client machines. Each machine has multiple threads that are continuously sending requests in an infinite loop. There are 2 agents; 1 agent for each client machine. The client threads send the requests to the corresponding agent for their machine.  

#### Client Implementation
Each client machine creates 500 threads. Each thread loops infinitely, continuously sending requests to the corresponding agent for this client machine. The client waits for a response for each request sent. In case a reply is not received, the thread times out after 3 seconds, and proceeds to send the next request, ignoring the failed one.

#### Client Agents
The agents act as a middleware between the clients and the servers. We have one agent for each client machine. Each agent receives the requests from the clients and propagates them to the server. This is where our load balancing algorithm takes place. 

#### Load Balancing
We implemented a round robin load balancing algorithm to fairly distribute the load among the servers. Round robin is a static scheduling algorithm that chooses the server to send the request to according to a predefined order. In other words, the servers are listed in a specific order and client requests are sent to them in that order. When the list is exhausted, the load balancer returns to the beginning of the list. 

#### Agent Implementation
The agent has two processes: a parent process and a child process. In the parent process, a thread is created for each client request it receives. A separate thread is responsible for concatenating the client port number to the client request data. This is done to be able to send the reply back to the same client that sent the request. After adding the port number, this thread adds the modified request data to a queue. Another separate thread then sends each request in the queue to the server, according to the round robin load balancer. The load balancer only considers the currently active servers; it excludes any servers that are down at the moment of sending the request. The parent process gets the information regarding which servers are active or inactive from the child process.
The child process has a thread that receives the replies from the servers and places them into a queue. This thread also receives the “Server down” and “Server up” messages (mentioned in the message formats table below). Upon receipt, the thread updates the list of active servers, and sends it to the parent process through an OS pipe. Another thread in the child process then takes each reply from the queue, removes the extra concatenated client port number and sends it back to the client. The usage of queues in the agent does not create an overhead, as sending/receiving through the socket happens sequentially.

### Servers

#### Network Topology

As mentioned above, our system has 3 servers. We have designed our servers to have a ring topology, where each server knows the addresses of the servers on its “right” and “left” sides. The main advantage of using a ring topology is that there is no need to store global information about all the servers in the network, which is efficient in terms of memory usage. This global information could be saved in every server in the network, which creates a huge storage overhead, especially in large networks. Also, this can cause inconsistency issues, in regards to updating the global tables in all the servers. To avoid the issues of inconsistency, this data could be stored in a centralized control unit, however, this creates a single point of failure for the system.Taking into account the aforementioned limitations, we chose to implement the ring topology for the servers. 
Server Implementation
Each server can exchange messages with its 2 neighboring servers and with the agents. To test the capability of the system to tolerate server failure, we used a distributed election algorithm to periodically elect a server to go down for a certain amount of time. There is a separate thread in the server that is responsible for starting the election process every 60 seconds. The server contains an infinite loop that is continuously receiving messages from both the agents and the neighboring servers. The servers keep track of the addresses of the agents that have sent requests to that server so far. When a message is received, the server first checks the source address of this message. If the source address is not a neighboring server address, then this message is a client request. Therefore, the server first updates its local list of agent addresses, then proceeds to process the request. The client requests are of the format [1,2,3,xxx,xx], where xxxxx represents the client port number. Processing the request in our simulation is basically reversing the array [1,2,3] that was received, sending [3,2,1], leaving the client port unchanged. So the reply is of the format [3,2,1,xxx,xx]. The reply is sent back to the source agents. 
The other case is that the message source is one of the neighboring servers. In this case, the message is concerned with the distributed elections algorithm which is discussed in the following subsection. 

#### Fault Tolerance
To assess the behavior of our system in case of a server failure, we need to periodically choose one of the three servers to go down for a period of time. We implemented a distributed elections algorithm to do so. In each server, a separate thread is responsible for a repeating timer that fires every 60 seconds, triggering the start of a new election process. When the election first starts, the server sends an election message to its 2 neighboring addresses. When a server receives an election message from one of its neighbors, the message is stored in an array ELECTION, such that the election messages from the left neighbor and the right neighbor are stored in ELECTION[0] and ELECTION[1] respectively. The election message is of the format “e,<addr>,<val>”; where “e” indicates that it is an election message, <addr> is the address of the currently nominated server, and <val> is the value for this specific server. This value is used as the election criteria; this is the value upon which we could decide which of the servers would win each election. Since our project is implemented in the scope of a LAN, and only has 3 servers, it is not realistic and/or possible to collect meaningful values for the election criteria. Typically, the election criteria would consist of network connection strength, distance from other servers, etc. Therefore, we generate a random value in each server to be used as the election criteria. This value is regenerated differently in every election round. 
For every election message received, the server updates the ELECTION array. The server then checks the values of the nominated servers in the array and if its own address is not yet nominated, it starts to consider itself too to be elected. The server then compares the value of the servers in the election messages with each other and with its own value. The server with the highest value would be the one it would nominate next. The server will then send an election message with the chosen server address and value in the aforementioned format. If the nominated server is one of the ones already in the ELECTION array, then the new election message will only be sent to one server, which is the neighbor that is not the same server that had sent the election message to begin with. In other words, when an election message is received from neighbor A, the server compares the values, decides on the current nominee, and sends it to neighbor B. If the server itself were not already one of the nominated servers and wants to nominate itself (because it has the highest value), it would send the new election message with its address and value to neighbors A and B, regardless of who had sent election messages before.
The election process ends when one of the servers receives election messages from both neighbors that are nominating the same server. This means that all servers have agreed on who should be elected next. When this happens, the server sends an election result message to its neighbors. This message has the format “r, <addr>, <val>”, where “r” indicates that this is an election result message, <addr> is the address of the elected server and <val> is the value of the server when it was elected. 
When the election is done, each server checks if they are the one that was elected, and if that is the case, they go down for 15 seconds. When a server goes down, it sends a message to all the agents in its local array and to its 2 neighboring servers that it is going down. This message is just one byte “d”. This is one of the points where the agents update their arrays of the currently active servers. Within these 15 seconds of down time, the server does not respond to any messages. When the down time is done, the server generates a new random value for the next election round and informs the agents and its 2 neighbors that it is back up. This message is “u”. The agents, again, update their arrays of currently active servers. 
Since the periodic timers in the servers are all local, they are not synchronized. Therefore, to avoid having overlapping elections, before a server can start a new election, it must check its local list of active servers to make sure that the previously elected server is now back up.

  ## Results 
  
The results below are based on data collected every 10 seconds for around 5 hours and 40 minutes. So, there are 2058 data points for each metric. The data collected at each point is an accumulation of all previous data.
  
### Client-Side
  
The response time is calculated as the time interval between the moment the sender first sends the request, until a response is received. The average response time is the sum of the response times divided by the number of responses received. The success rate is the number of responses received divided by the number of requests sent. 

|                                                | Average Response Time (ms)| Success Rate (%)  |
|-----------------------                         |---------------------------| -----:|
| Client 1 (500 threads)                         | 14.74                     | 99.68 |
| Client 2 (500 threads)                         | 14.18                     | 99.70 |
| Average Client in the System (all 1000 threads)|14.45                      | 99.69 |
 
 The plots below show how these metrics varied over time. 
![alt text](https://github.com/mohamedayman15069/Distributed-System-Project-using-Rust/blob/main/Images/1.png)
![alt text](https://github.com/mohamedayman15069/Distributed-System-Project-using-Rust/blob/main/Images/2.png)
![alt text](https://github.com/mohamedayman15069/Distributed-System-Project-using-Rust/blob/main/Images/3.png)

### Agent-Side 
  
|                  | Server 1 | Server2 | Server 3|
|------------------|----------|---------| -----:|
| Agent 1          | 0.332    |0.341    | 0.326 |
| Agent            | 0.334    |0.342    | 0.324 |
| Average of agents| 0.333    |0.342    | 0.325 |
  
  The plot below shows the number of requests sent to each server by both agents over time.
![alt text](https://github.com/mohamedayman15069/Distributed-System-Project-using-Rust/blob/main/Images/4.png)
  
### Server-Side
Even though the load balancer has distributed the load fairly over the servers, packets may be lost in the network for many reasons. Therefore, below we show the actual load on each of the servers.

  |             Ratio of Requests Received by Each Server             | 
  | Server 1 | server 2 | server 3|
  |0.322     | 0.341    | 0.326   |
  
  
![alt text](https://github.com/mohamedayman15069/Distributed-System-Project-using-Rust/blob/main/Images/5.png)
