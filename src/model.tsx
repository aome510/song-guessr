type User = {
  display_name: string;
};

type Playlist = {
  id: string;
  name: string;
  owner: User;
};

type Choice = {
  name: string;
  preview_url: string;
};

type Question = {
  choices: Array<Choice>;
  ans_id: number;
};

type UserData = {
  id: string;
  name: string;
};

export type { User, Playlist, Choice, Question, UserData };
