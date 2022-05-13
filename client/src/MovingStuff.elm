module MovingStuff exposing (..)

import Html exposing (..)
import Html.Attributes as Attr
import Html.Events exposing (..)

-- Tick ->
--     Debug.log "tick" ( model, Task.perform identity (Task.succeed HideQuadstew) )
--
-- , Attr.classList
--     [ ( "animate__animated", model.animationState == Clicked )
--     , ( "animate__pulse", model.animationState == Pulse )
--     , ( "animate__infinite", model.animationState == Pulse )
--     , ( "animate__headShake", model.animationState == Clicked )
--     , ( "animate__slow", model.animationState == Clicked )
--     ]


enterImage : String -> Html msg
enterImage p =
    div
        [ Attr.classList
            [ ( "animate__animated", True )
            , ( "animate__bounceInDown", True )
            ]
        ]
        [ Html.img [ Attr.src p ] [] ]


leaveImage : String -> Html msg
leaveImage p =
    div
        [ Attr.classList
            [ ( "animate__animated", True )
            , ( "animate__bounceOutUp", True )
            ]
        ]
        [ Html.img [ Attr.src p ] [] ]


-- subscriptions : Model -> Sub Msg
-- subscriptions model =
--     case model.animationState of
--         None ->
--             Sub.none
--
--         Leaving ->
--             Sub.none
--
--         Clicked ->
--             Time.every 3000 (\_ -> HideQuadstew)
