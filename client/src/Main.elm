port module Main exposing (..)

import Browser
import Html exposing (Html, button, div, h1, input, li, text, ul)
import Html.Attributes exposing (placeholder, type_, value)
import Html.Events exposing (on, onClick, onInput)
import Json.Decode as D exposing (Decoder, andThen, fail, field, int, map2, string, succeed)
import Json.Encode exposing (Value)
import Json.Decode exposing (oneOf)


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


type PossibleCommands
    = DisplayCmd String
    | OtherCmd
    | ErrorCmd

teej : Decoder PossibleCommands
teej = 
    oneOf [
        point
        ]

-- (field "variant" string) returns "display"
-- (field "data" string) returns "hello world"


type alias PartialMsg =
    { variant : String, data : String }


point : Decoder PartialMsg
point =
    map2 PartialMsg
        (field "variant" string)
        (field "data" string)

pointTeej : Decoder PossibleCommands
pointTeej =
    point



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

-- displayDecoder : Decoder PossibleCommands
-- displayDecoder = 
    -- field "data" testing123
    -- andThen testing123
    -- map2
    --     (field "data" string)

    -- if x.variant == "display" then
    --     displayFunc(x.data as DisplayCmd)
    -- else if x.data == "other" then
    --     otherFunc(x.data as OtherCmd)
    -- else
    --     error
    -- end


-- testing123 x =
--     case D.decodeString (field "data" string) x of
--         Ok msg -> succeed (DisplayCmd msg)
--         _ -> succeed (DisplayCmd "OK")



ebnIsCool : Decoder PossibleCommands
ebnIsCool =
    point
        |> andThen ebnPart2


ebnPart2 : PartialMsg -> Decoder PossibleCommands
ebnPart2 pMsg =
    case pMsg of
        { variant, data } ->
            case variant of
                "display" ->
                    succeed (DisplayCmd data)

                _ ->
                    fail "second level"



-- _ -> fail "Not today baby"
-- dependsStart : Decoder DependingOnWhatYouMean
-- dependsStart =
--   field "variant" string
--     |> andThen dependsWhatYouMeanHelp
--
-- dependsWhatYouMeanHelp : String -> Decoder DependingOnWhatYouMean
-- dependsWhatYouMeanHelp remaining =
--   case remaining of
--     "display" ->
--         case D.decodeString (field "data" string) remaining of
--             Ok msg -> succeed (DisplayCmd msg)
--             Err _ -> fail <| remaining
--         -- succeed (DisplayCmd "Yup, pretend this is from JSON")
--         -- succeed (DisplayCmd (field "data" string))
--
--     "not display" ->
--         succeed OptionTwo
--
--     _ ->
--       fail <|
--         "Trying to decode info, but version "
--         ++ remaining ++ " is not supported."
--
-- Use the `sendMessage` port when someone presses ENTER or clicks
-- the "Send" button. Check out index.html to see the corresponding
-- JS where this is piped into a WebSocket.
--


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
                    D.decodeString jerseyMilker message
            in
            case something of
                Ok (DisplayCmd msgToDisplay) ->
                    ( { model | messages = model.messages ++ [ msgToDisplay ] }, Cmd.none )

                Ok ErrorCmd ->
                    ( { model | messages = model.messages ++ [ "oh yeah, part 2" ] }, Cmd.none )

                Err errMsg ->
                    ( { model | messages = model.messages ++ [ Debug.toString errMsg ] }, Cmd.none )



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
