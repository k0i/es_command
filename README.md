![wslhost exe 01 08 2022 6_53_39](https://user-images.githubusercontent.com/100127291/182046871-1c422460-b70f-490b-8994-1aeb388c257e.png)

# es_command

**es_command** is a wrapper command for [evans[ktr0731/evans]](https://github.com/ktr0731/evans) that enables dynamic json parse.

```shell
es 'multiple json file path'
```

> **Warning**

- #### This command depends on evans. Please verify that you already installed it.

- #### This is one of my Hobby-Project and **still under working**.

- #### It supports only UnaryRequest and can be used with only reflection at present.

# Installation

## [Github Releases](https://github.com/k0i/es_command/releases)
### linux

```shell
wget https://github.com/k0i/es_command/releases/download/#{version}/es-linux-musl-x86 -O es
```

### Mac OS x

```shell
curl https://github.com/k0i/es_command/releases/download/#{version}/es-macos-x86 > es
```

> **Info**
Please place es file within a directory specified by the PATH environment variable.


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
