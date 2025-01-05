import { Button, Flex, Heading, List } from "@chakra-ui/react";
import { EndedGameState } from "./model";
import { post } from "./utils";

const GameResults: React.FC<{
  room: string;
  state: EndedGameState;
}> = ({ room, state }) => {
  return (
    <Flex direction="column" gap="4">
      <Heading size="3xl">Game Results</Heading>
      <List.Root>
        {state.users.map((user) => (
          <List.Item key={user.name}>
            {user.name}: {user.score}
          </List.Item>
        ))}
      </List.Root>
      <Button
        onClick={() => {
          post(`/api/room/${room}/reset`, {});
        }}
      >
        New Game
      </Button>
    </Flex>
  );
};

export default GameResults;
