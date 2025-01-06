import { useNavigate } from "react-router-dom";
import UserForm from "./components/UserForm";
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
    <Flex direction="column" alignItems="center" gap="4">
      <Heading size="6xl">Song Guessr</Heading>
      <Text textStyle="2xl">Welcome, {user.name}</Text>
      <Button onClick={newRoom}>Create a new room</Button>
    </Flex>
  );
}

export default HomePage;
