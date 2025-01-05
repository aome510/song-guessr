import { useEffect, useMemo, useState } from "react";
import { useParams } from "react-router-dom";
import UserForm from "./UserForm";
import { getUserData } from "./utils";
import {
  EndedGameState,
  PlayingGameState,
  User,
  WaitingGameState,
} from "./model";
import WaitingRoom from "./WaitingRoom";
import Game from "./Game";
import GameResults from "./GameResults";
import { Flex, Heading } from "@chakra-ui/react";

function getWsUri(room_id: string, user: User): string {
  const url = new URL(
    `/api/room/${room_id}?user_id=${user.id}&user_name=${user.name}`,
    window.location.origin,
  );
  url.protocol = url.protocol == "https:" ? "wss:" : "ws:";
  return url.href;
}

function Room() {
  const { id } = useParams();
  const user = useMemo(() => getUserData(), []);
  const [ws, setWs] = useState<WebSocket | null>(null);
  const [type, setType] = useState<string>("");
  const [state, setState] = useState<
    PlayingGameState | WaitingGameState | null
  >(null);

  if (id === undefined) {
    throw new Error("Room ID is undefined");
  }

  useEffect(() => {
    if (user === null) {
      return;
    }

    const ws = new WebSocket(getWsUri(id, user));

    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      setType(data.type);
      setState(data);
    };

    setWs(ws);

    return () => {
      ws.close();
    };
  }, [id, user]);

  if (user === null) {
    return <UserForm />;
  }

  if (ws === null) {
    return <div>Connecting to the server...</div>;
  }

  const content = () => {
    if (type == "Waiting") {
      return <WaitingRoom state={state as WaitingGameState} id={id} />;
    } else if (type == "Playing") {
      return (
        <Game state={state as PlayingGameState} ws={ws} user={user} room={id} />
      );
    } else if (type == "Ended") {
      return <GameResults state={state as EndedGameState} room={id} />;
    }
  };

  return (
    <Flex w="100vw" justifyContent="center">
      <Flex
        h="100vh"
        direction="column"
        justifyContent="center"
        gap="4"
        maxW="75%"
      >
        <Heading size="3xl">Room {id}</Heading>
        {content()}
      </Flex>
    </Flex>
  );
}

export default Room;
