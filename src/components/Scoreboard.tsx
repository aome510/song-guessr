import { Flex, Heading, List } from "@chakra-ui/react";
import { UserGameState, UserSubmission } from "../model";

const Scoreboard: React.FC<{
  title: string;
  users: Array<UserGameState>;
  submissions?: Array<UserSubmission>;
}> = ({ title, users, submissions }) => {
  users.sort((a, b) => b.score - a.score);
  return (
    <Flex direction="column" gap="2">
      <Heading size="xl">{title}</Heading>
      <List.Root>
        {users.map((user) => {
          const score = submissions?.find(
            (s) => s.user_name === user.name,
          )?.score;

          return (
            <List.Item key={user.name}>
              {user.name}: {user.score} {score !== undefined && `(+${score})`}
            </List.Item>
          );
        })}
      </List.Root>
    </Flex>
  );
};

export default Scoreboard;
