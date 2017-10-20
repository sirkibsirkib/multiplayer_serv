# multiplayer_serv

2nd attempt at a multiplayer-centric rust game. This time, the engine is built around a different premise: The server makes the game-logic decisions that impact other players.
It seems pretty obvious when I say it like that, but its actually now the only viable way to do it.

Now, clients send requetsts

## Structure

Built to accomodate both single-player and multiplayer, the system has a modular design centered around a divide between the user-end that does not impact the game state globally ("client side"),
and the game-logic side that does ("server side"). In multiplayer, these two sides communicate over a network as you would expect. For singleplayer, their communication channels are directly coupled. From the engines' perspective, there is no perceptible difference.

![GitHub Logo](https://github.com/sirkibsirkib/multiplayer_serv/blob/master/idea.png)
