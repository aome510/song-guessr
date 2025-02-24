import { useEffect, useMemo, useState } from "react";
import { useParams } from "react-router-dom";
import UserForm from "./components/UserForm";
import { get, getUserData } from "./utils";
import {
  EndedGameState,
  PlayingGameState,
  User,
  WaitingForNextQuestionState,
  WaitingGameState,
} from "./model";
import WaitingRoom from "./WaitingRoom";
import Game from "./Game";
import GameResults from "./GameResults";
import { Flex, Heading } from "@chakra-ui/react";
import QuestionResults from "./QuestionResults";

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
  const [isOwner, setIsOwner] = useState(false);

  useEffect(() => {
    async function checkOwner() {
      if (user !== null) {
        const response = await get(
          `/api/room/${id}/is_owner?user_id=${user.id}`,
        );
        setIsOwner(await response.json());
      }
    }
    checkOwner();
  }, [user, id]);

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
      if (
        [
          "WaitingForGame",
          "Playing",
          "WaitingForNextQuestion",
          "Ended",
        ].includes(data.type)
      ) {
        setType(data.type);
        setState(data);
      }
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
    switch (type) {
      case "WaitingForGame":
        return (
          <WaitingRoom
            state={state as WaitingGameState}
            room={id}
            user={user}
            isOwner={isOwner}
          />
        );
      case "Playing":
        return (
          <Game
            state={state as PlayingGameState}
            ws={ws}
            user={user}
            room={id}
            isOwner={isOwner}
          />
        );
      case "WaitingForNextQuestion":
        return <QuestionResults state={state as WaitingForNextQuestionState} />;
      case "Ended":
        return (
          <GameResults
            state={state as EndedGameState}
            room={id}
            user={user}
            isOwner={isOwner}
          />
        );
    }
  };

  return (
    <Flex direction="column" justifyContent="center" gap="4" maxW="75%">
      <Heading size="3xl">Room {id}</Heading>
      {content()}
    </Flex>
  );
}

export default Room;
