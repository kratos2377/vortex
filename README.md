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
- [ ] Use async redis pool



## Todays plan
- [ ] Work on architecture
- [x] Implement certois so that it can consume/produce events
- [ ] Start with basic, produce an event to show user is online to all the friends of the user
- [ ] Think about staking and integrating wallets/ how can we let users spectate match

## APIS to add
- [ ] Users can stake money in games
- [ ] Users can see/invite his online friends to play games
- [ ] Users can see online friends
- [ ] Few games like chess and poker can be simple games or staked game

## Structure

| Codebase              |      Description          |
| :-------------------- | :-----------------------: |
| [messier](crates/messier)    |    Axum API and WS Events |
| [cerotis](crates/cerotis)    |  Consume Kafka Events and Process          |
| [ton](crates/ton)            |      Entity Models        |
| [migration](crates/migration)|       Migration Logic     |
| [saggitarius](crates/saggitarius)        |   Mongo Kafka Streams Consumer        |
| [helix](crates/helix)        |   Kafka Schema Publisher |
| [orion](crates/orion)        |   Common Models for mongodb and some kafka events |