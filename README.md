<img width="782" alt="Снимок экрана 2022-08-01 055158" src="https://user-images.githubusercontent.com/100127291/182045182-2b353e3d-5ef4-4bd9-8ca0-450e16770d5e.png">

# es_command

**es_command** is a wrapper command for [evans[ktr0731/evans]](https://github.com/ktr0731/evans) that enables dynamic json parse.

```shell
es 'multiple json file path'
```

> **Warning**

- #### This command depends on evans. Please verify that you already installed it.

- #### This is one of my Hobby-Project and **still under working**.

- #### It supports only UnaryRequest and can be used with only reflection at present.

# Usage

## Create a json file.

- Example
  Given that first response is following structure.

```json
{
  "greetMessage": {
    "greetId": 1,
    "name": "test",
    "nameJp": "テスト"
  }
}
```

You can reference the response field using special delimiter `$$` as shown below.

```json
[
  {
    "name": "createGreet",
    "method": "api.v1.GreetService.CreateGreet",
    "body": {
      "name": "test",
      "nameJp": "テスト"
    }
  },
  {
    "method": "api.v1.GreetService.GetGreet",
    "body": {
      "greet_id": "$$createGreet.greetMessage.greetId"
    }
  }
]
```
