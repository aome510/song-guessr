import { useEffect, useMemo, useState } from "react";
import { GameState, Question, UserGameState } from "./model.tsx";
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
  const [users, setUsers] = useState<Array<UserGameState>>([]);
  const [questionId, setQuestionId] = useState<number>(-1);
  const [question, setQuestion] = useState<Question | null>(null);
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

        if (state.question_id !== questionId) {
          setQuestionId(state.question_id);
          setQuestion(state.question);
          setSelectedChoice(null);
          audio.src = state.question.song_url;
        }

        if (state.users !== users) {
          setUsers(state.users);
        }
      } else if (data.type == "GameEnded") {
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
  }, [ws, audio, userData, questionId, users, navigate]);

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

  if (question === null) {
    return <div>Loading...</div>;
  }

  return (
    <div>
      <h2>Question {questionId + 1}</h2>
      {question.choices.map((choice, index) => (
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
        {users.map((user) => (
          <li key={user.name}>
            {user.name}: {user.score}
          </li>
        ))}
      </ul>
    </div>
  );
}

export default Game;
