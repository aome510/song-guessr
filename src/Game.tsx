import { useEffect, useMemo, useState } from "react";
import { Question } from "./model.tsx";
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
  const [currentQuestion, setCurrentQuestion] = useState<Question | null>(null);
  const [currentQuestionId, setCurrentQuestionId] = useState<number>(0);
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
      if (data.type === "Question") {
        setCurrentQuestionId(data.id);
        const question = data.question as Question;
        setCurrentQuestion(question);
        audio.src = question.choices[question.ans_id].preview_url;
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
    if (currentQuestion === null) {
      return;
    }
    if (selectedChoice === currentQuestion.ans_id) {
      alert("Correct!");
    } else {
      alert(
        "Incorrect! The correct song is " +
          currentQuestion.choices[currentQuestion.ans_id].name,
      );
    }
    ws.send(
      JSON.stringify({
        type: "NextQuestion",
      }),
    );
  };

  if (userData === null) {
    return <UserForm />;
  }

  if (currentQuestion === null) {
    return <div>Loading...</div>;
  }

  return (
    <div>
      <h1>Question {currentQuestionId + 1}</h1>
      {currentQuestion.choices.map((choice, index) => (
        <button
          key={index}
          type="button"
          onClick={() => handleChoiceSubmit(index)}
          style={{
            backgroundColor: "gray",
            color: "white",
            margin: "5px",
            padding: "10px",
          }}
        >
          {choice.name}
        </button>
      ))}
    </div>
  );
}

export default Game;
