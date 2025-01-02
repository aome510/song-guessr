import { useEffect, useMemo, useState } from "react";
import { GameState } from "./model.tsx";
import { useNavigate, useParams } from "react-router-dom";
import { getUserData } from "./utils.tsx";
import UserForm from "./UserForm.tsx";

function getWsUri(id: string): string {
  const url = new URL(`api/game/${id}`, window.location.origin);
  url.protocol = url.protocol == "https:" ? "wss:" : "ws:";
  return url.href;
}

function Game() {
  const userData = getUserData();
  const [gameState, setGameState] = useState<GameState | null>(null);
  const [selectedChoice, setSelectedChoice] = useState<number | null>(null);
  const navigate = useNavigate();
  const { id } = useParams();

  if (id === undefined) {
    throw new Error("Game ID is undefined");
  }

  const ws = useMemo(() => new WebSocket(getWsUri(id)), [id]);

  const audio = useMemo(() => new Audio(), []);

  useEffect(() => {
    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      if (data.type === "GameState") {
        const state = data as GameState;
        setGameState(state);
        setSelectedChoice(null);
        audio.src = state.question.song_url;
      } else if (data.type == "GameEnded") {
        alert("Game ended!");
        navigate("/");
      }
    };

    ws.onopen = () => {
      if (userData === null) {
        return;
      }

      ws.send(
        JSON.stringify({
          type: "UserJoined",
          name: userData.name,
          id: userData.id,
        }),
      );
    };
  }, [ws, audio, userData, navigate]);

  useEffect(() => {
    audio.autoplay = true;
    audio.volume = 0.5;
    return () => {
      audio.pause();
    };
  }, [audio]);

  const handleChoiceSubmit = (selectedChoice: number) => {
    setSelectedChoice(selectedChoice);
    ws.send(
      JSON.stringify({
        type: "UserSubmitted",
        user_id: userData?.id,
        choice: selectedChoice,
      }),
    );
  };

  if (userData === null) {
    return <UserForm />;
  }

  if (gameState === null) {
    return <div>Loading...</div>;
  }

  return (
    <div>
      <h2>Question {gameState.question_id + 1}</h2>
      {gameState.question.choices.map((choice, index) => (
        <button
          key={index}
          type="button"
          onClick={() => handleChoiceSubmit(index)}
          disabled={selectedChoice !== null}
          style={{
            backgroundColor: selectedChoice === index ? "blue" : "gray",
            color: "white",
            margin: "5px",
            padding: "10px",
          }}
        >
          {choice.name}
        </button>
      ))}
      <h2>Scoreboard</h2>
      <ul>
        {gameState.users.map((user) => (
          <li key={user.name}>
            {user.name}: {user.score}
          </li>
        ))}
      </ul>
    </div>
  );
}

export default Game;
