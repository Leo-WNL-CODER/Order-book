//optimization checks 
----------code repetition---->write a function
----------check the price is valid for the given tick size
-----------------------
#right now writing a program to handle the buy orders 
---------------------------

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
--------------------
Sell order handling
--------------------

    ---->checking if the buyer order list is empty
        |
        |--yes-->then just push the order in the seller list 
        |
        |--no--->fetch the max bid
            |
            |---->loop starts if max_bid >= current ask  at that price level
            |
            |---->if the remaining sell quantity is not equal to zero 
                |
                |->store the remaining quant in the sell order list
        

--------------------------------
#
---------------------------------
to make the engine process orders fast the conversion of the tick size and price should happen outside of the engine also
since tick size and price are in floating point form, converting them inside the engine or limitorderbook functions may result in precision error ....
Now the question may arise that it can cause slippage as it has to go through another gateway api and then the engine.
but here is the catch the conversion takes same amount of time inside the matching engine and in the api gateway.So if we keep the conversions inside the engine which will be running on the single thread and If the engine stops to parse a JSON string, extract an f64, multiply it, and round it, it takes (for example) 50 nanoseconds.
If 10,000 orders arrive at the exact same time, the 10,000th order has to wait for the engine to do that float math 9,999 times before it finally gets matched. That queue buildup will cause massive slippage. so the 10000th order will likely take 10000*50 ns which will cause slippage.

