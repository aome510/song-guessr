import { User, WaitingGameState } from "./model";
import React from "react";
import Search from "./Search";
import { Heading, List, Flex, Text } from "@chakra-ui/react";

const WaitingRoom: React.FC<{
  state: WaitingGameState;
  room: string;
  user: User;
  isOwner: boolean;
}> = ({ state, room, user, isOwner }) => {
  return (
    <Flex gap="4" direction="column">
      <div>
        <Heading size="xl">Users</Heading>
        <List.Root>
          {state.users.map((user, i) => (
            <List.Item key={i}>{user.name}</List.Item>
          ))}
        </List.Root>
      </div>
      {isOwner && <Search room={room} user={user} />}
      {!isOwner && (
        <Text textStyle="md">Waiting for the owner to start the game...</Text>
      )}
    </Flex>
  );
};

export default WaitingRoom;
