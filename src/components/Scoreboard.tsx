import { Flex, Heading, Table } from "@chakra-ui/react";
import { UserGameState, UserSubmission } from "../model";

const Scoreboard: React.FC<{
  title: string;
  users: Array<UserGameState>;
  submissions?: Array<UserSubmission>;
}> = ({ title, users, submissions }) => {
  users.sort((a, b) => b.score - a.score);
  return (
    <Flex direction="column" gap="4" p="4" borderRadius="lg">
      <Heading size="xl">{title}</Heading>
      <Table.Root size="lg">
      <Table.Header>
        <Table.Row bg="gray.100">
        <Table.ColumnHeader fontWeight="bold">Player</Table.ColumnHeader>
        <Table.ColumnHeader fontWeight="bold" textAlign="right">Score</Table.ColumnHeader>
        </Table.Row>
      </Table.Header>
      <Table.Body>
        {users.map((user, index) => {
        const score = submissions?.find(
          (s) => s.user_name === user.name,
        )?.score;

        return (
          <Table.Row 
          key={user.name}
          bg={index % 2 === 0 ? "white" : "gray.50"}
          _hover={{ bg: "blue.50" }}
          >
          <Table.Cell fontWeight="medium">{user.name}</Table.Cell>
          <Table.Cell textAlign="right">
            <Flex gap="2" justify="flex-end" align="center">
            <span>{user.score}</span>
            {score !== undefined && score > 0 && (
              <span style={{ color: "green", fontWeight: "bold" }}>
              (+{score})
              </span>
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
