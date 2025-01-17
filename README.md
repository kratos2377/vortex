# Vortex


Multiplayer Backend Server for my games like chess and Monopoly , (Games which can run on TCP)


To use this its good to send something like game event where each event will tell us which user made the move and which room should we broadcast it to and then we can parse event according to the game which is in the event

## All Crates
- [ ] Refactor code

## Messier
- [ ] Dont send user token in reponse of auth APIs and set in cookie, make required changes in client side as well

## Nebula
- [ ] Add WS Layer For Nebula
- [ ] Add Solana Transaction Logic To create bets on DEX
- [ ] Add APIs for DEXs to see stock updates/ Portfolio / Swap Trading /Spot Trading


## Hyperion
- [ ] Add Events to update game bets

## Structure

| Codebase              |      Description          |
| :-------------------- | :-----------------------: |
| [messier](crates/messier)    |    Axum API and WS Events |
| [cerotis](crates/cerotis)    |  Consume Specific Kafka Events and process           |
| [ton](crates/ton)            |      Entity Models        |
| [migration](crates/migration)|       Migration Logic     |
| [orion](crates/orion)        |   Common Models for mongodb , WS and some kafka events |
| [nebula](crates/nebula)        |  API to Stake In Games and find status of those stakes + APIs for DEX as well |
| [hyperion](crates/hyperion)        | Settle bets |