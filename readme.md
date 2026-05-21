So , the basic functions of adding,executing and canceling the orders has been added.

Initially we were using vector to store the orders at the price level but canceling it would have taken O(n)  since we have to wanted O(1) access time we shifted to using the more like slab allocator structure .

this image below explains the data structres used and what is each function responsible for

![Click](image.png)