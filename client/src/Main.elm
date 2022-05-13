port module Main exposing (..)

import Browser
import Html exposing (Html, button, div, h1, input, li, text, ul)
import Html.Attributes exposing (placeholder, type_, value)
import Html.Events exposing (on, onClick, onInput)
import Json.Decode as D exposing (Decoder)
import MovingStuff exposing (enterImage, leaveImage)
import Process
import Task


main : Program () Model Msg
main =
    Browser.element
        { init = init
        , update = update
        , view = view
        , subscriptions = subscriptions
        }


type ImgStatus
    = None
    | Visible String
    | Leaving String


type alias Model =
    { draft : String
    , messages : List String
    , img : ImgStatus
    }


init : () -> ( Model, Cmd Msg )
init _ =
    ( { draft = "", messages = [], img = None }
    , Cmd.none
    )


type Msg
    = DraftChanged String
    | Recv String
    | Send
    | HideImage String


type PossibleCommands
    = DisplayCmd String
    | ShowImg String
    | ErrorCmd


{-| Data looks something like
{ "variant": "display", "data": ... }
{ "variant": "showimg", "data": ... }
-}
wolfadex : Decoder PossibleCommands
wolfadex =
    D.field "variant" D.string
        |> D.andThen
            (\variant ->
                case variant of
                    "display" ->
                        D.field "data" displayCmdDecoder

                    "show-image" ->
                        D.field "data" showImgDecoder

                    _ ->
                        D.fail "some error"
            )


displayCmdDecoder : Decoder PossibleCommands
displayCmdDecoder =
    D.string
        |> D.map DisplayCmd


showImgDecoder : Decoder PossibleCommands
showImgDecoder =
    D.string
        |> D.map ShowImg


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        DraftChanged draft ->
            ( { model | draft = draft }
            , Cmd.none
            )

        Send ->
            ( { model | draft = "" }
            , sendMessage model.draft
            )

        HideImage p ->
            ( { model | img = Leaving p }, Cmd.none )

        -- {
        --     "variant": "display",
        --     "data": {
        --         "message": resp,
        --     },
        -- }
        --
        Recv message ->
            let
                something =
                    D.decodeString wolfadex message
            in
            case something of
                Ok (DisplayCmd msgToDisplay) ->
                    ( { model | messages = model.messages ++ [ msgToDisplay ] }, Cmd.none )

                Ok (ShowImg imgToDisplay) ->
                    ( { model | img = Visible imgToDisplay }
                    , Process.sleep 1000
                        |> Task.map (\_ -> HideImage imgToDisplay)
                        |> Task.perform identity
                    )

                Ok ErrorCmd ->
                    ( { model | messages = model.messages ++ [ "oh yeah, part 2" ] }, Cmd.none )

                Err errMsg ->
                    ( { model | messages = model.messages ++ [ Debug.toString errMsg ] }, Cmd.none )


port sendMessage : String -> Cmd msg


port messageReceiver : (String -> msg) -> Sub msg



-- SUBSCRIPTIONS
-- Subscribe to the `messageReceiver` port to hear about messages coming in
-- from JS. Check out the index.html file to see how this is hooked up to a
-- WebSocket.


subscriptions : Model -> Sub Msg
subscriptions _ =
    messageReceiver Recv


view : Model -> Html Msg
view model =
    div []
        [ h1 [] [ text "Echo Chat" ]
        , ul []
            (List.map (\msg -> li [] [ text msg ]) model.messages)
        , input
            [ type_ "text"
            , placeholder "Draft"
            , onInput DraftChanged
            , on "keydown" (ifIsEnter Send)
            , value model.draft
            ]
            []
        , button [ onClick Send ] [ text "Send" ]
        , case model.img of
            None ->
                div [] [ Html.text "hello" ]

            Visible p ->
                div [] [ enterImage p ]

            Leaving p ->
                div [] [ leaveImage p ]
        ]


ifIsEnter : msg -> D.Decoder msg
ifIsEnter msg =
    D.field "key" D.string
        |> D.andThen
            (\key ->
                if key == "Enter" then
                    D.succeed msg

                else
                    D.fail "some other key"
            )
