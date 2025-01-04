import { useEffect, useMemo, useState } from "react";
import { PlayingGameState, Question, User, UserGameState } from "./model.tsx";

const Game: React.FC<{
  ws: WebSocket;
  state: PlayingGameState;
  user: User;
}> = ({ ws, state, user }) => {
  const [users, setUsers] = useState<Array<UserGameState>>([]);
  const [questionId, setQuestionId] = useState<number>(-1);
  const [question, setQuestion] = useState<Question | null>(null);
  const [selectedChoice, setSelectedChoice] = useState<number | null>(null);
  const audio = useMemo(() => new Audio(), []);

  useEffect(() => {
    if (state.question_id !== questionId) {
      setQuestionId(state.question_id);
      setQuestion(state.question);
      setSelectedChoice(null);
      audio.src = state.question.song_url;
    }

    if (state.users !== users) {
      setUsers(state.users);
    }
  }, [state, questionId, users, audio]);

  useEffect(() => {
    audio.autoplay = true;
    audio.volume = 0.5;
    return () => {
      audio.pause();
      audio.src = "";
    };
  }, [audio]);

  const handleChoiceSubmit = (selectedChoice: number) => {
    setSelectedChoice(selectedChoice);
    ws.send(
      JSON.stringify({
        type: "UserSubmitted",
        user_id: user.id,
        choice: selectedChoice,
      }),
    );
  };

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
};

export default Game;
