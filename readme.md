//optimization checks 
----------code repetition---->write a function

right now i am writing a program to handle the buy orders 

--------------------
Buyer order handling
--------------------

    ---->checking if the seller order list is empty
        |
        |--yes-->then just push the order in the buyer list 
        |
        |--no--->fetch the min ask
            |
            |---->if min_ask <= current bid 
                |
                |--->loop starts at that price level
            |
            |---->if min ask>current bid
                |
                |--->store the current bid in the BUY ORDER LIST