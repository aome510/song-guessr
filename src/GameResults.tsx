import { Button, Flex } from "@chakra-ui/react";
import { EndedGameState, User } from "./model";
import { put } from "./utils";
import Scoreboard from "./components/Scoreboard";

const GameResults: React.FC<{
  room: string;
  state: EndedGameState;
  user: User;
  isOwner: boolean;
}> = ({ room, state, user, isOwner }) => {
  return (
    <Flex direction="column" gap="4">
      <Scoreboard title="Results" users={state.users} />
      {isOwner && (
        <Flex direction="column" gap="2">
          <Button
            onClick={() => {
              put(`/api/room/${room}/restart`, { user_id: user.id });
            }}
          >
            Restart Game
          </Button>
          <Button
            onClick={() => {
              put(`/api/room/${room}/reset`, { user_id: user.id });
            }}
          >
            Back to Lobby
          </Button>
        </Flex>
      )}
    </Flex>
  );
};

export default GameResults;
