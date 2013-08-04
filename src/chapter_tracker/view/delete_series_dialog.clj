(ns chapter-tracker.view.delete-series-dialog (:gen-class)
  (:require clojure.string)
  (:use chapter-tracker.view.tools)
  (:use chapter-tracker.model)
)
(import
  '(java.awt GridBagLayout GridBagConstraints Dimension)
  '(javax.swing JFrame JLabel JTextField JOptionPane)
)

(defn create-delete-series-dialog[series-record on-confirmation-function]
  (create-frame {:title "Delete Series"}
                (let [
                      confirmation-textbox (JTextField. 20)
                     ]
                  (add-with-constraints (JLabel. "This will delete the series:") (gridx 0) (gridy 0))
                  (add-with-constraints (JLabel. (.toString series-record)) (gridx 0) (gridy 1))
                  (add-with-constraints (JLabel. "Type \"yes\" in the textbox to configrm") (gridx 0) (gridy 2))
                  (add-with-constraints confirmation-textbox (gridx 0) (gridy 3))
                  (add-with-constraints (action-button "CONFIRM"
                                                       (if (= "yes" (.getText confirmation-textbox))
                                                         (do
                                                           (on-confirmation-function)
                                                           (.dispose frame))
                                                         (JOptionPane/showMessageDialog
                                                           nil
                                                           (str "Type \"yes\" if you really want to delete\n" (.toString series-record))
                                                           "Error - unable to delete" JOptionPane/ERROR_MESSAGE)
                                                       )
                                              )
                                        (gridx 0) (gridy 4) (fill GridBagConstraints/HORIZONTAL))
                )
  )
)
