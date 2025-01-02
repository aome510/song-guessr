import { useState } from "react";
import { v4 as uuidv4 } from "uuid";

function UserForm() {
  const [userName, setUserName] = useState("");

  return (
    <div>
      <h2>Please provide your username</h2>
      <form
        onSubmit={() => {
          localStorage.setItem("userId", uuidv4());
          localStorage.setItem("userName", userName);
        }}
      >
        <label>
          Username:
          <input
            type="text"
            value={userName}
            onChange={(e) => {
              setUserName(e.target.value);
            }}
            required
          />
        </label>
        <button type="submit">Submit</button>
      </form>
    </div>
  );
}

export default UserForm;
