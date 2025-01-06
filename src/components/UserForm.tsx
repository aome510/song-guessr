import { Button, Field, Flex, Heading, Input } from "@chakra-ui/react";
import { useState } from "react";
import { v4 as uuidv4 } from "uuid";

function UserForm() {
  const [userName, setUserName] = useState("");

  return (
    <Flex direction="column" gap="4">
      <Heading size="xl">Please provide your username</Heading>
      <form
        onSubmit={() => {
          localStorage.setItem("userId", uuidv4());
          localStorage.setItem("userName", userName);
        }}
      >
        <Field.Root>
          <Field.Label>Username</Field.Label>
          <Input
            type="text"
            onChange={(e) => {
              setUserName(e.target.value);
            }}
            required
          />
        </Field.Root>
        <Button type="submit" mt="2">
          Submit
        </Button>
      </form>
    </Flex>
  );
}

export default UserForm;
