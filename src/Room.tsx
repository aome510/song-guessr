import { useEffect, useMemo, useState } from "react";
import { useParams } from "react-router-dom";
import UserForm from "./UserForm";
import { getUserData } from "./utils";
import { PlayingGameState, UserData, WaitingGameState } from "./model";
import WaitingRoom from "./WaitingRoom";

function getWsUri(room_id: string, userData: UserData): string {
  const url = new URL(
    `api/room/${room_id}?user_id=${userData.id}&user_name=${userData.name}`,
    window.location.origin,
  );
  url.protocol = url.protocol == "https:" ? "wss:" : "ws:";
  return url.href;
}

function Room() {
  const { id } = useParams();
  const userData = useMemo(() => getUserData(), []);
  const [type, setType] = useState<string>("");
  const [state, setState] = useState<
    PlayingGameState | WaitingGameState | null
  >(null);

  if (id === undefined) {
    throw new Error("Room ID is undefined");
  }

  useEffect(() => {
    if (userData === null) {
      return;
    }

    const ws = new WebSocket(getWsUri(id, userData));

    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);

      console.log(data);

      if (data.type == "Waiting") {
        setType("Waiting");
        setState(data);
      }
    };

    return () => {
      ws.close();
    };
  }, [id, userData]);

  const content = () => {
    if (type == "Waiting") {
      return <WaitingRoom state={state as WaitingGameState} />;
    }
  };

  if (userData === null) {
    return <UserForm />;
  }

  return (
    <div>
      <h1>Room {id}</h1>
      <div>{content()}</div>
    </div>
  );
}

export default Room;
