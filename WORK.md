## Nebula and Hyperion work

Nebula will get events when game is over (game_id , session_id together will make a game_bet record)

Once done it will generate events and store in Redis cache  with a time
Once time is hit the redis will generate a event which will be read by hyperion


Hyperion will have a queue initially of 200 
    -> We will also keep a network of bots/executors (which will settle all the bets)
        -> Each bot will read one event from queue and execute -> if execution was successful  then we generate a event back to nebula updating all the recrds
        -> If execution was not successful we store the event again in cache with extended time


- [ ] Work on nebula and execute queries to get records (1 game and 1 session related) 
- [ ] Store these in cache with time -> (After that time is reached redis will generate events that will be read by hyperion)


- [ ] For hyperion build the queue system first
