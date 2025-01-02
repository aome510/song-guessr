type SpotifyUser = {
  display_name: string;
};

type Playlist = {
  id: string;
  name: string;
  owner: SpotifyUser;
};

type Choice = {
  name: string;
};

type Question = {
  choices: Array<Choice>;
  song_url: string;
};

type UserData = {
  id: string;
  name: string;
};

type UserGameState = {
  name: string;
  score: number;
};

type GameState = {
  question: Question;
  question_id: number;
  users: Array<UserGameState>;
};

export type { Playlist, UserData, GameState };
