# multiplayer_serv

2nd attempt at a multiplayer-centric rust game. This time, the engine is built around a different premise: The server makes the game-logic decisions that impact other players.
It seems pretty obvious when I say it like that, but its actually now the only viable way to do it.

There are two different versions of the same game represented in a complete game session: The client's game and the server's game. The work required is essentially partitioned amongst the two according to the logic of : "What affects global state?". So UI, inventory management and other jazz like that is all done clientside. entity movement, weather effects and creation and destruction of world objects is done serverside.

The two halves communicate via messages, with each delegating work not within their jurisdiction to the other. Conceptually, the server has the final word.

In single player, this distinction still exists, but their communication interface is trivial, rather than being existing over a network. 

## Structure

Built to accomodate both single-player and multiplayer, the system has a modular design centered around a divide between the user-end that does not impact the game state globally ("client side"),
and the game-logic side that does ("server side"). In multiplayer, these two sides communicate over a network as you would expect. For singleplayer, their communication channels are directly coupled. From the engines' perspective, there is no perceptible difference.

![GitHub Logo](https://github.com/sirkibsirkib/multiplayer_serv/blob/master/idea.png)
