import { useNavigate } from "react-router-dom";
import UserForm from "./UserForm";
import { getUserData, post } from "./utils";
import { Button, Flex, Heading, Text } from "@chakra-ui/react";

function HomePage() {
  const user = getUserData();
  const navigate = useNavigate();

  if (user === null) {
    return <UserForm />;
  }

  const newRoom = async () => {
    try {
      const response = await post("/api/room", {
        user_id: user.id,
      });
      const data = await response.json();
      navigate(`/room/${data.room_id}`);
    } catch (err) {
      console.error(err);
    }
  };

  return (
    <Flex h="100vh" direction="column">
      <Heading size="6xl">Song Guessr</Heading>
      <Text textStyle="3xl">Welcome, {user.name}</Text>
      <Button onClick={newRoom}>Create a new room</Button>
    </Flex>
  );
}

export default HomePage;
