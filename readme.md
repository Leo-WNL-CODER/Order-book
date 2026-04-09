//optimization checks 
----------code repetition---->write a function
----------check the price is valid for the given tick size

#right now writing a program to handle the buy orders 

--------------------
Buyer order handling
--------------------

    ---->checking if the seller order list is empty
        |
        |--yes-->then just push the order in the buyer list 
        |
        |--no--->fetch the min ask
            |
            |---->loop starts if min_ask <= current bid  at that price level
            |
            |---->if the remaining buy quantity is not equal to zero 
                |
                |->store the remaining quant in the buy order list
        