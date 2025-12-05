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
        <Flex direction="column" gap="3" mt="4">
          {state.users.map((user, i) => (
            <Flex
              key={i}
              align="center"
              p="3"
              borderRadius="md"
              borderWidth="1px"
            >
              <Flex
                w="8"
                h="8"
                bg="blue.500"
                borderRadius="full"
                align="center"
                justify="center"
                fontWeight="bold"
                mr="3"
              >
                {user.name.charAt(0).toUpperCase()}
              </Flex>
              <Text fontWeight="medium">{user.name}</Text>
            </Flex>
          ))}
        </Flex>
      </div>
      {isOwner && <Search room={room} user={user} />}
      {!isOwner && (
        <Text textStyle="md">Waiting for the owner to start the game...</Text>
      )}
    </Flex>
  );
};

export default WaitingRoom;
