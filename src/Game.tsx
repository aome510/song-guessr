import { useEffect, useMemo, useState } from "react";
import { Question } from "./model.tsx";
import { get } from "./utils.tsx";
import { useParams } from "react-router-dom";

function Game() {
  const [currentQuestion, setCurrentQuestion] = useState<Question | null>(null);
  const { id } = useParams();

  const audio = useMemo(() => new Audio(), []);

  useEffect(() => {
    audio.autoplay = true;
    audio.volume = 0.5;
    return () => {
      audio.pause();
    };
  }, [audio]);

  useEffect(() => {
    const getGameState = async () => {
      try {
        const response = await get(`http://localhost:8000/game/${id}`);
        const data = await response.json();
        const question = data.question as Question;
        setCurrentQuestion(question);
        audio.src = question.choices[question.ans_id].preview_url;
      } catch (err) {
        console.error(err);
      }
    };

    getGameState();
  }, [audio, id]);

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
  };

  if (currentQuestion === null) {
    return <div>Loading...</div>;
  }

  return (
    <div>
      {currentQuestion.choices.map((choice, index) => (
        <button
          key={choice.name}
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
