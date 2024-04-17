#!/bin/bash

# Replace with your Zookeeper container name (if different)
ZOOKEEPER_HOST=zookeeper

# Topic names (modify as needed)
# TOPICS=("user" "game" "user_game_events")
sh -c 'kafka-topics --create --bootstrap-server localhost:9092 --replication-factor 1 --partitions 2 --topic game'
# for TOPIC in "${TOPICS[@]}"; do
#   kafka-topics --create --bootstrap-servers $ZOOKEEPER_HOST:2181 --topic $TOPIC --partitions 2 --replication-factor 1
#   if [ $? -eq 0 ]; then
#     echo "Topic '$TOPIC' created successfully."
#   else
#     echo "Error creating topic '$TOPIC'."
#   fi
# done

echo "All topic creation processes complete."