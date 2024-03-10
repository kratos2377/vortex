# Vortex


Multiplayer Backend Server for my games like chess and Monopoly , (Games which can run on TCP)


To use this its good to send something like game event where each event will tell us which user made the move and which room should we broadcast it to and then we can parse event according to the game which is in the event

## Todo
- [ ] Add kafka/grpc for some realtime streaming for some use cases
    - [ ] Events so that users can know their friends are online
    - [ ] Users can spectate games and the games which are played on stake mode can be staked by other players in the given time limit
    - [ ] Events for realtime updates for spectating users

- [ ] Users can manage their sols by investing in DEXes
- [ ] Socket disconnect functions

