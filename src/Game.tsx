import { useEffect, useMemo, useState } from "react";
import { Question } from "./model.tsx";
import { makeApiRequest } from "./utils.tsx";
import { useNavigate } from "react-router-dom";

function Game() {
  const [questionId, setQuestionId] = useState<number>(-1);
  const [questions, setQuestions] = useState<Array<Question>>([]);

  const queryParams = new URLSearchParams(window.location.search);
  const playlistId = queryParams.get("playlist_id");
  const numQuestions = queryParams.get("num_questions") || "15";

  const audio = useMemo(() => new Audio(), []);

  useEffect(() => {
    audio.autoplay = true;
    audio.volume = 0.5;
    return () => {
      audio.pause();
    };
  }, [audio]);

  const navigate = useNavigate();

  useEffect(() => {
    const fetchQuestions = async () => {
      if (playlistId !== "") {
        try {
          const response = await makeApiRequest(
            `http://localhost:8000/questions/${playlistId}?num_questions=${numQuestions}`,
          );
          const data = await response.json();
          console.assert(
            data instanceof Array,
            "Expected an array of questions",
          );
          setQuestions(data);
          setQuestionId(0);
          audio.src = data[0].choices[data[0].ans_id].preview_url;
        } catch (err) {
          console.error(err);
        }
      }
    };

    fetchQuestions();
  }, [audio, numQuestions, playlistId]);

  const handleChoiceSubmit = (selectedChoice: number) => {
    const question = questions[questionId];
    if (selectedChoice === question.ans_id) {
      alert("Correct!");
    } else {
      alert(
        "Incorrect! The correct song is " +
          question.choices[question.ans_id].name,
      );
    }
    // Move to the next question
    if (questionId + 1 < questions.length) {
      setQuestionId(questionId + 1);
      const next_question = questions[questionId + 1];
      audio.src = next_question.choices[next_question.ans_id].preview_url;
    } else {
      alert("Quiz completed!");
      navigate("/");
    }
  };

  if (questionId === -1) {
    return <div>Loading...</div>;
  }

  return (
    <div>
      <h2>Question {questionId + 1}</h2>
      {questions[questionId].choices.map((choice, index) => (
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
