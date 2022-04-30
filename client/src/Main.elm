port module Main exposing (..)

import Browser
import Html exposing (Html, button, div, h1, input, li, text, ul)
import Html.Attributes exposing (placeholder, type_, value)
import Html.Events exposing (on, onClick, onInput)
import Json.Decode exposing (Decoder, andThen, fail, field, int, map2, oneOf, string, succeed, Value)

import Json.Decode as D
import Json.Encode as E


main : Program () Model Msg
main =
    Browser.element
        { init = init
        , update = update
        , view = view
        , subscriptions = subscriptions
        }


type alias Model =
    { draft : String
    , messages : List String
    }


init : () -> ( Model, Cmd Msg )
init _ =
    ( { draft = "", messages = [] }
    , Cmd.none
    )


type Msg
    = DraftChanged String
    | Recv String
    | Send


-- { "variant": "display", "data": "hello world" }
-- { "variant": "increment", "data": 5 }

type PossibleCommands
    = DisplayCmd String
    | OtherCmd Int
    | ErrorCmd

displayCmdDecoder : Decoder PossibleCommands
displayCmdDecoder = D.map DisplayCmd (D.field "data" D.string)

otherCmdDecoder : Decoder PossibleCommands
otherCmdDecoder = D.map OtherCmd (D.field "data" D.int)

dependsStart : D.Decoder PossibleCommands
dependsStart =
  D.field "variant" D.string
    |> D.andThen dependsWhatYouMeanHelp

somethingElse : Decoder PossibleCommands
somethingElse =
    D.andThen dependsWhatYouMeanHelp (D.field "variant" D.string)

dependsWhatYouMeanHelp : String -> D.Decoder PossibleCommands
dependsWhatYouMeanHelp remaining =
  case remaining of
    "display" -> displayCmdDecoder
    "other" -> otherCmdDecoder
    _ -> D.fail "blabla"

-- teej : Decoder PossibleCommands
-- teej =
--     oneOf
--         [ displayCmd
--         ]
--
-- -- "display", data should be a string
-- displayCmd : Decoder PossibleCommands
-- displayCmd =
--     map2 makeDisplayCommand
--         (field "variant" string)
--         (field "data" string)
--
-- variantDisplayDecoder : Decoder String
-- variantDisplayDecoder =
--     (field "variant" string)
--
--
-- litString : Decoder String
-- litString =
--     case decodeString (field "variant" string) of
--         Ok "display" -> succeed "variant"
--         _ -> fail "variant"
--
-- makeDisplayCommand : String -> String -> PossibleCommands
-- makeDisplayCommand variant msg =
--     case variant of
--         "display" -> DisplayCmd msg
--         _ -> ErrorCmd
--
-- alwaysSuceeed =
--     succeed (DisplayCmd "Hello")
--
--
-- (field "variant" string) returns "display"
-- (field "data" string) returns "hello world"
--
--
-- type alias PartialMsg =
--     { variant : String, data : String }
--
--
-- point : Decoder PartialMsg
-- point =
--     map2 PartialMsg
--         (field "variant" string)
--         (field "data" string)
--
-- pointTeej : Decoder PossibleCommands
-- pointTeej =
--     point
-- decodeString point """{ "x": 3, "y": 4 }""" == Ok { x = 3, y = 4 }


jerseyMilker : Decoder PossibleCommands
jerseyMilker =
    field "variant" string
        |> andThen msgMultiplex


msgMultiplex : String -> Decoder PossibleCommands
msgMultiplex variant =
    case variant of
        -- "display" ->
        --     displayDecoder
        -- "other" ->
        _ ->
            fail <|
                "hey dawg, plz use good names"



type alias DisplayMsg =
    { type_ : String, data : String }


type alias ResponseMsg =
    DisplayMsg


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
                    D.decodeString dependsStart message
            in
            case something of
                Ok (DisplayCmd msgToDisplay) ->
                    ( { model | messages = model.messages ++ [ msgToDisplay ] }, Cmd.none )

                Ok ErrorCmd ->
                    ( { model | messages = model.messages ++ [ "oh yeah, part 2" ] }, Cmd.none )

                Err errMsg ->
                    ( { model | messages = model.messages ++ [ Debug.toString errMsg ] }, Cmd.none )
                _ -> 
                    ( { model | messages = model.messages ++ [ "oh yeah, fallthrough" ] }, Cmd.none )



-- view : Int -> Html Msg
-- view model =
--     div []
--         [ button [ onClick Decrement ] [ text "-" ]
--         , div [] [ text (String.fromInt model) ]
--         , button [ onClick Increment ] [ text "+" ]
--         ]
--


port sendMessage : String -> Cmd msg


port messageReceiver : (String -> msg) -> Sub msg



-- SUBSCRIPTIONS
-- Subscribe to the `messageReceiver` port to hear about messages coming in
-- from JS. Check out the index.html file to see how this is hooked up to a
-- WebSocket.
--


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
