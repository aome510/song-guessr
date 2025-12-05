import { Flex, Heading, Table, Text } from "@chakra-ui/react";
import { UserGameState, UserSubmission } from "../model";

const Scoreboard: React.FC<{
  title: string;
  users: Array<UserGameState>;
  submissions?: Array<UserSubmission>;
}> = ({ title, users, submissions }) => {
  users.sort((a, b) => b.score - a.score);
  return (
    <Flex direction="column" gap="2" borderRadius="lg" w="full">
      <Heading size="xl">{title}</Heading>
      <Table.Root size="lg">
        <Table.Header>
          <Table.Row bg="gray.100">
            <Table.ColumnHeader fontWeight="bold">Player</Table.ColumnHeader>
            <Table.ColumnHeader fontWeight="bold" textAlign="right">
              Score
            </Table.ColumnHeader>
          </Table.Row>
        </Table.Header>
        <Table.Body>
          {users.map((user) => {
            const score = submissions?.find(
              (s) => s.user_name === user.name,
            )?.score;

            return (
              <Table.Row key={user.name} _hover={{ bg: "blue.50" }}>
                <Table.Cell fontWeight="medium">{user.name}</Table.Cell>
                <Table.Cell textAlign="right">
                  <Flex gap="2" justify="flex-end" align="center">
                    <Text>{user.score}</Text>
                    {score !== undefined && score > 0 && (
                      <Text color="green.500" fontWeight="bold">
                        (+{score})
                      </Text>
                    )}
                  </Flex>
                </Table.Cell>
              </Table.Row>
            );
          })}
        </Table.Body>
      </Table.Root>
    </Flex>
  );
};

export default Scoreboard;
