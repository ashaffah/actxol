# ACTXOL

#### READ OPTION

**1. Get user by username**

```js
"http://127.0.0.1:8080/api/user/{username}",
  {
    method: "GET",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
  };
```

**2. Get all users**

```js
"http://127.0.0.1:8080/api/users",
  {
    method: "GET",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
  };
```

Available query params : `?page=1&per_page=10&search=hello`

**3. Get qr**

```js
"http://127.0.0.1:8080/api/qr?data={data}",
  {
    method: "GET",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
  };
```

#### WRITE OPTION

**1. Add user**

```js
"http://127.0.0.1:8080/api/add_user",
  {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
    body: JSON.stringify({
      first_name: "hello",
      last_name: "world",
      username: "hello",
      email: "hello@gmail.com",
    }),
  };
```

**2. Update user**

```js
"http://127.0.0.1:8080/api/user/{username}",
  {
    method: "PUT",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
    body: JSON.stringify({
      first_name: "hello",
      last_name: "world",
      username: "hello",
      email: "hello@gmail.com",
    }),
  };
```

**3. Delete user**

```js
"http://127.0.0.1:8080/api/user/{username}",
  {
    method: "DELETE",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
  };
```
