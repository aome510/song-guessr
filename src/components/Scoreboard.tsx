import { Flex, Heading, List } from "@chakra-ui/react";
import { UserGameState } from "../model";

const Scoreboard: React.FC<{ title: string; users: Array<UserGameState> }> = ({
  title,
  users,
}) => {
  return (
    <Flex direction="column" gap="2">
      <Heading size="xl">{title}</Heading>
      <List.Root>
        {users.map((user) => {
          return (
            <List.Item key={user.name}>
              {user.name}: {user.score}
            </List.Item>
          );
        })}
      </List.Root>
    </Flex>
  );
};

export default Scoreboard;
