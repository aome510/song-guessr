import { WaitingGameState } from "./model";
import React from "react";
import Search from "./Search";
import { Heading, List, Flex } from "@chakra-ui/react";

const WaitingRoom: React.FC<{ state: WaitingGameState; id: string }> = ({
  state,
  id,
}) => {
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
      <Search room={id} />
    </Flex>
  );
};

export default WaitingRoom;
